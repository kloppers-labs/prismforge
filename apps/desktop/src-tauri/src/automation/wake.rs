use prismforge_core::DeviceRef;
use prismforge_core::automation::{
    RuleRun, RunFailure, Trigger, WakeReady, WakeTarget, wait_after_wake,
};
use prismforge_core::session::SessionHandle;

use crate::automation::run::{failed, is_live, record, run_rule};
use crate::automation::{WAKE_APPLY_LIMIT, now_ms};
use crate::publisher::ViewSink;
use crate::shell::Shell;

/// Reaches the session through the shell for each step, so the wait never holds a session handle.
struct ShellTarget<'a, S: ViewSink> {
    shell: &'a Shell<S>,
}

impl<S: ViewSink> WakeTarget for ShellTarget<'_, S> {
    async fn is_live(&self) -> bool {
        self.shell
            .snapshot()
            .await
            .is_some_and(|session| is_live(&session.connection))
    }

    async fn reconnect(&self) {
        if let Ok(connect) = self.shell.queue(SessionHandle::connect).await {
            let _ = connect.await;
        }
    }

    async fn devices(&self) -> Option<Vec<DeviceRef>> {
        self.shell
            .queue(SessionHandle::refresh)
            .await
            .ok()?
            .await
            .ok()?;
        let snapshot = self.shell.snapshot().await?;
        Some(
            snapshot
                .devices
                .iter()
                .filter(|device| device.available)
                .filter_map(|device| device.identity.reference().cloned())
                .collect(),
        )
    }
}

/// `current` is false once a newer sleep started; the wake rule then does not run.
pub(crate) async fn run_wake<S: ViewSink>(
    shell: &Shell<S>,
    before: &[(DeviceRef, String)],
    current: impl Fn() -> bool,
) -> Option<RuleRun> {
    let (handle, kind) = shell.current().await;
    let profiles = shell.stores.profiles(kind).await.ok()?;
    profiles
        .rules()
        .await
        .rule(Trigger::Wakes)
        .due(Trigger::Wakes)?;
    let Some(handle) = handle else {
        let run = RuleRun {
            trigger: Trigger::Wakes,
            at_ms: now_ms(),
            outcome: failed(RunFailure::NotConnected),
        };
        record(shell, &profiles, run.clone()).await;
        return Some(run);
    };
    let session = handle.session_id();
    let _ = handle.resync().await;
    drop(handle);
    let refs: Vec<DeviceRef> = before.iter().map(|(device, _)| device.clone()).collect();
    let ready = wait_after_wake(&ShellTarget { shell }, &refs).await;
    let same_session = shell
        .current()
        .await
        .0
        .is_some_and(|handle| handle.session_id() == session);
    if !current() || !same_session {
        return None;
    }
    let missing = match ready {
        WakeReady::NeverBack { waited } => {
            let run = RuleRun {
                trigger: Trigger::Wakes,
                at_ms: now_ms(),
                outcome: failed(RunFailure::OpenRgbNotBack {
                    waited_ms: waited.as_millis() as u64,
                }),
            };
            record(shell, &profiles, run.clone()).await;
            return Some(run);
        }
        WakeReady::Ready { missing, .. } => before
            .iter()
            .filter(|(device, _)| missing.contains(device))
            .map(|(_, name)| name.clone())
            .collect(),
    };
    if let Some(handle) = shell.current().await.0 {
        shell.checks.refresh(&handle).await;
    }
    run_rule(shell, Trigger::Wakes, WAKE_APPLY_LIMIT, missing).await
}
