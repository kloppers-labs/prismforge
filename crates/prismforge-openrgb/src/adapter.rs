use prismforge_core::{
    BackendError, BackendEvent, BackendInfo, BackendOrigin, ConnectionState, DeviceCapabilities,
    DeviceDescriptor, DeviceIdentity, DeviceIssue, DeviceRef, Inventory, InventoryDevice,
    IssueReason, KnownDevice, LossCause, Reconciliation, Rgb, RgbBackend, StateReason, reconcile,
};
use tokio::time::timeout;

use crate::client::{ClientConfig, ClientError, MIN_PROTOCOL_VERSION, Notification, OpenRgbClient};
use crate::connector::Connector;
use crate::translate::describe;

pub struct OpenRgbBackend<C: Connector> {
    connector: C,
    config: ClientConfig,
    client: Option<OpenRgbClient<C::Stream>>,
    state: ConnectionState,
    known: Vec<KnownDevice>,
    inventory: Option<Inventory>,
    runtime_ids: Vec<u32>,
    stale: bool,
}

struct Scan {
    devices: Vec<(u32, DeviceDescriptor)>,
    issues: Vec<DeviceIssue>,
    vanished: bool,
}

enum Target {
    Device,
    Zone(usize),
}

impl<C: Connector> OpenRgbBackend<C> {
    pub fn new(connector: C, config: ClientConfig) -> Self {
        Self {
            connector,
            config,
            client: None,
            state: ConnectionState::Disconnected { reason: None },
            known: Vec::new(),
            inventory: None,
            runtime_ids: Vec::new(),
            stale: true,
        }
    }

    pub fn protocol_version(&self) -> Option<u32> {
        self.client.as_ref().map(OpenRgbClient::protocol_version)
    }

    fn absorb_notifications(&mut self) {
        while let Some(notification) = self
            .client
            .as_mut()
            .and_then(OpenRgbClient::poll_notification)
        {
            self.apply(notification);
        }
    }

    fn apply(&mut self, notification: Notification) -> BackendEvent {
        match notification {
            Notification::DeviceListUpdated => {
                self.stale = true;
                self.update_state();
                BackendEvent::InventoryChanged
            }
            Notification::Closed(error) => {
                let cause = loss_cause(&error);
                self.lose_connection(cause);
                BackendEvent::ConnectionLost {
                    cause,
                    detail: error.to_string(),
                }
            }
        }
    }

    fn lose_connection(&mut self, cause: LossCause) {
        self.client = None;
        self.stale = true;
        self.state = ConnectionState::Disconnected {
            reason: Some(StateReason::ConnectionLost { cause }),
        };
    }

    fn unreachable(&mut self, detail: String) -> BackendError {
        self.state = ConnectionState::Disconnected {
            reason: Some(StateReason::ServerUnreachable),
        };
        BackendError::Unreachable { detail }
    }

    fn fail(&mut self, error: ClientError) -> BackendError {
        self.lose_connection(loss_cause(&error));
        backend_error(error)
    }

    fn update_state(&mut self) {
        if self.client.is_none() {
            return;
        }
        let mut reasons = Vec::new();
        if self.stale && self.inventory.is_some() {
            reasons.push(StateReason::InventoryStale);
        }
        if let Some(inventory) = &self.inventory {
            if !inventory.issues.is_empty() {
                reasons.push(StateReason::ControllerDataRejected {
                    controllers: inventory.issues.len(),
                });
            }
            let ambiguous = inventory
                .devices
                .iter()
                .filter(|device| matches!(device.identity, DeviceIdentity::Ambiguous { .. }))
                .count();
            if ambiguous > 0 {
                reasons.push(StateReason::AmbiguousDevices { devices: ambiguous });
            }
        }
        self.state = if reasons.is_empty() {
            ConnectionState::Connected
        } else {
            ConnectionState::Degraded { reasons }
        };
    }

    fn adopt(&mut self, scan: Scan, reconciliation: Reconciliation) {
        let mut identities: Vec<Option<DeviceIdentity>> = vec![None; scan.devices.len()];
        for matched in reconciliation.matched {
            identities[matched.current] = Some(DeviceIdentity::Matched(matched.reference));
        }
        for new in reconciliation.new {
            identities[new.current] = Some(DeviceIdentity::New(new.reference));
        }
        for group in reconciliation.ambiguous {
            for current in group.current {
                identities[current] = Some(DeviceIdentity::Ambiguous {
                    candidates: group.candidates.clone(),
                });
            }
        }
        let mut devices = Vec::with_capacity(scan.devices.len());
        self.runtime_ids.clear();
        for ((runtime_id, descriptor), identity) in scan.devices.into_iter().zip(identities) {
            let identity = identity.expect("reconciliation classifies every current device");
            if let Some(reference) = identity.reference() {
                self.remember(reference, &descriptor);
            }
            self.runtime_ids.push(runtime_id);
            devices.push(InventoryDevice {
                identity,
                descriptor,
            });
        }
        let generation = self
            .inventory
            .as_ref()
            .map_or(1, |previous| previous.generation + 1);
        self.inventory = Some(Inventory {
            generation,
            devices,
            missing: reconciliation.missing,
            issues: scan.issues,
        });
        self.stale = scan.vanished;
    }

    fn remember(&mut self, reference: &DeviceRef, descriptor: &DeviceDescriptor) {
        let fingerprint = descriptor.fingerprint();
        match self
            .known
            .iter_mut()
            .find(|known| &known.reference == reference)
        {
            Some(known) => known.fingerprint = fingerprint,
            None => self.known.push(KnownDevice {
                reference: reference.clone(),
                fingerprint,
            }),
        }
    }

    fn lookup(&self, device: &DeviceRef) -> Result<usize, BackendError> {
        let inventory = self
            .inventory
            .as_ref()
            .ok_or(BackendError::StaleInventory)?;
        if let Some(index) = inventory
            .devices
            .iter()
            .position(|entry| entry.identity.reference() == Some(device))
        {
            return Ok(index);
        }
        let device = device.clone();
        Err(if inventory.is_ambiguous(&device) {
            BackendError::AmbiguousDevice { device }
        } else if inventory.missing.contains(&device) {
            BackendError::DeviceMissing { device }
        } else {
            BackendError::UnknownDevice { device }
        })
    }

    /// Runtime ID for `device` in the current generation; refuses anything not provably that device.
    fn resolve(
        &mut self,
        device: &DeviceRef,
        target: &Target,
        colors: Option<usize>,
    ) -> Result<u32, BackendError> {
        self.absorb_notifications();
        if self.client.is_none() {
            return Err(BackendError::NotConnected);
        }
        if self.stale {
            return Err(BackendError::StaleInventory);
        }
        let index = self.lookup(device)?;
        let descriptor = &self
            .inventory
            .as_ref()
            .expect("lookup found an inventory")
            .devices[index]
            .descriptor;
        if !descriptor.capabilities.has_direct_mode() {
            return Err(BackendError::DirectModeUnavailable {
                device: device.clone(),
            });
        }
        let expected = match target {
            Target::Device => descriptor.led_count,
            Target::Zone(zone) => {
                descriptor
                    .zones
                    .get(*zone)
                    .ok_or(BackendError::ZoneOutOfRange {
                        device: device.clone(),
                        zone: *zone,
                        zone_count: descriptor.zones.len(),
                    })?
                    .led_count
            }
        };
        if let Some(actual) = colors.filter(|&actual| actual != expected) {
            return Err(BackendError::ColorCountMismatch {
                device: device.clone(),
                expected,
                actual,
            });
        }
        Ok(self.runtime_ids[index])
    }

    async fn scan(client: &mut OpenRgbClient<C::Stream>) -> Result<Scan, ClientError> {
        let mut scan = Scan {
            devices: Vec::new(),
            issues: Vec::new(),
            vanished: false,
        };
        for id in client.controller_ids().await? {
            match client.controller_data(id).await {
                Ok(data) => scan.devices.push((id, describe(&data))),
                Err(ClientError::ControllerData {
                    controller_id,
                    error,
                }) => scan.issues.push(DeviceIssue {
                    runtime_id: controller_id,
                    reason: if error.is_inconsistency() {
                        IssueReason::InconsistentData
                    } else {
                        IssueReason::MalformedData
                    },
                    detail: error.to_string(),
                }),
                Err(ClientError::ControllerNotFound { .. }) => scan.vanished = true,
                Err(error) => return Err(error),
            }
        }
        Ok(scan)
    }
}

impl<C: Connector> RgbBackend for OpenRgbBackend<C> {
    fn connection_state(&self) -> &ConnectionState {
        &self.state
    }

    fn info(&self) -> BackendInfo {
        BackendInfo {
            origin: BackendOrigin::OpenRgb,
            address: self.connector.address(),
            protocol_version: self.protocol_version(),
            server_name: self.client.as_ref().and_then(OpenRgbClient::server_name),
        }
    }

    async fn connect(&mut self) -> Result<(), BackendError> {
        self.disconnect();
        self.state = ConnectionState::Connecting;
        let stream = match timeout(self.config.request_timeout, self.connector.connect()).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(error)) => return Err(self.unreachable(error.to_string())),
            Err(_) => return Err(self.unreachable("connection attempt timed out".into())),
        };
        match OpenRgbClient::connect(stream, self.config.clone()).await {
            Ok(client) => {
                self.client = Some(client);
                self.update_state();
                Ok(())
            }
            Err(error) => {
                let reason = match &error {
                    ClientError::UnsupportedProtocol { server_version } => {
                        StateReason::UnsupportedProtocol {
                            server_version: *server_version,
                        }
                    }
                    other => StateReason::ConnectionLost {
                        cause: loss_cause(other),
                    },
                };
                self.state = ConnectionState::Disconnected {
                    reason: Some(reason),
                };
                Err(backend_error(error))
            }
        }
    }

    fn disconnect(&mut self) {
        self.client = None;
        self.stale = true;
        self.state = ConnectionState::Disconnected { reason: None };
    }

    async fn next_event(&mut self) -> Option<BackendEvent> {
        let notification = self.client.as_mut()?.next_notification().await;
        Some(self.apply(notification))
    }

    async fn refresh_inventory(&mut self) -> Result<&Inventory, BackendError> {
        self.absorb_notifications();
        let client = self.client.as_mut().ok_or(BackendError::NotConnected)?;
        let scan = match Self::scan(client).await {
            Ok(scan) => scan,
            Err(error) => return Err(self.fail(error)),
        };
        let fingerprints: Vec<_> = scan
            .devices
            .iter()
            .map(|(_, device)| device.fingerprint())
            .collect();
        let reconciliation = reconcile(&self.known, &fingerprints);
        self.adopt(scan, reconciliation);
        self.absorb_notifications();
        self.update_state();
        Ok(self.inventory.as_ref().expect("adopt stores an inventory"))
    }

    fn inventory(&self) -> Option<&Inventory> {
        self.inventory.as_ref()
    }

    fn capabilities(&self, device: &DeviceRef) -> Result<&DeviceCapabilities, BackendError> {
        let index = self.lookup(device)?;
        let inventory = self
            .inventory
            .as_ref()
            .ok_or(BackendError::StaleInventory)?;
        Ok(&inventory.devices[index].descriptor.capabilities)
    }

    async fn enable_direct_mode(&mut self, device: &DeviceRef) -> Result<(), BackendError> {
        let runtime_id = self.resolve(device, &Target::Device, None)?;
        let client = self.client.as_mut().ok_or(BackendError::NotConnected)?;
        match client.set_custom_mode(runtime_id).await {
            Ok(()) => Ok(()),
            Err(error @ ClientError::Rejected { .. }) => Err(backend_error(error)),
            Err(error) => Err(self.fail(error)),
        }
    }

    async fn write_device_frame(
        &mut self,
        device: &DeviceRef,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        let runtime_id = self.resolve(device, &Target::Device, Some(colors.len()))?;
        let client = self.client.as_mut().ok_or(BackendError::NotConnected)?;
        let result = client.update_leds(runtime_id, colors).await;
        result.map_err(|error| self.fail(error))
    }

    async fn write_zone_frame(
        &mut self,
        device: &DeviceRef,
        zone: usize,
        colors: &[Rgb],
    ) -> Result<(), BackendError> {
        let runtime_id = self.resolve(device, &Target::Zone(zone), Some(colors.len()))?;
        let client = self.client.as_mut().ok_or(BackendError::NotConnected)?;
        let result = client
            .update_zone_leds(runtime_id, zone as u32, colors)
            .await;
        result.map_err(|error| self.fail(error))
    }
}

fn loss_cause(error: &ClientError) -> LossCause {
    match error {
        ClientError::Timeout { .. } => LossCause::Timeout,
        ClientError::Closed { .. } => LossCause::ServerClosed,
        ClientError::Io { .. } => LossCause::Transport,
        ClientError::Abandoned => LossCause::CallAbandoned,
        _ => LossCause::ProtocolViolation,
    }
}

fn backend_error(error: ClientError) -> BackendError {
    match error {
        ClientError::UnsupportedProtocol { server_version } => BackendError::UnsupportedProtocol {
            server_version,
            required: MIN_PROTOCOL_VERSION,
        },
        ClientError::Timeout { operation, after } => BackendError::Timeout {
            operation: operation.into(),
            timeout_ms: after.as_millis() as u64,
        },
        ClientError::Io { .. } | ClientError::Closed { .. } | ClientError::Abandoned => {
            BackendError::ConnectionLost {
                cause: loss_cause(&error),
                detail: error.to_string(),
            }
        }
        ClientError::Rejected {
            packet_id, status, ..
        } => BackendError::Rejected {
            operation: format!("packet {packet_id}"),
            status,
        },
        other => BackendError::ProtocolViolation {
            detail: other.to_string(),
        },
    }
}
