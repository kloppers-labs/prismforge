use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::device::{DeviceKind, DeviceRef, ZoneKind};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub kind: DeviceKind,
    pub vendor: String,
    pub model: String,
    pub serial: Option<String>,
    pub location: Option<String>,
    pub zones: Vec<ZoneShape>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ZoneShape {
    pub name: String,
    pub kind: ZoneKind,
    pub leds_min: u32,
    pub leds_max: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnownDevice {
    pub reference: DeviceRef,
    pub fingerprint: DeviceFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Reconciliation {
    pub matched: Vec<IdentityMatch>,
    pub ambiguous: Vec<AmbiguousGroup>,
    pub missing: Vec<DeviceRef>,
    pub new: Vec<IdentityMatch>,
}

/// `current` is the position in the inventory slice passed to [`reconcile`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityMatch {
    pub reference: DeviceRef,
    pub current: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmbiguousGroup {
    pub candidates: Vec<DeviceRef>,
    pub current: Vec<usize>,
}

/// Matches current devices to known ones on identifying fields only; equal candidates stay unresolved.
pub fn reconcile(known: &[KnownDevice], current: &[DeviceFingerprint]) -> Reconciliation {
    let mut pass = Pass {
        known,
        current,
        open_known: (0..known.len()).collect(),
        open_current: (0..current.len()).collect(),
        result: Reconciliation::default(),
    };
    pass.pair_by(|fingerprint| Some(fingerprint.clone()));
    pass.pair_by(|fingerprint| {
        fingerprint.serial.as_ref().map(|_| DeviceFingerprint {
            location: None,
            ..fingerprint.clone()
        })
    });
    pass.finish()
}

struct Pass<'a> {
    known: &'a [KnownDevice],
    current: &'a [DeviceFingerprint],
    open_known: Vec<usize>,
    open_current: Vec<usize>,
    result: Reconciliation,
}

impl Pass<'_> {
    fn pair_by(&mut self, key: impl Fn(&DeviceFingerprint) -> Option<DeviceFingerprint>) {
        for (group_key, current) in group(&self.open_current, |index| key(&self.current[index])) {
            let candidates: Vec<usize> = self
                .open_known
                .iter()
                .copied()
                .filter(|&index| key(&self.known[index].fingerprint).as_ref() == Some(&group_key))
                .collect();
            if candidates.is_empty() {
                continue;
            }
            self.open_known.retain(|index| !candidates.contains(index));
            self.open_current.retain(|index| !current.contains(index));
            if let ([known], [current]) = (candidates.as_slice(), current.as_slice()) {
                self.result.matched.push(IdentityMatch {
                    reference: self.known[*known].reference.clone(),
                    current: *current,
                });
            } else {
                self.result.ambiguous.push(AmbiguousGroup {
                    candidates: candidates
                        .iter()
                        .map(|&index| self.known[index].reference.clone())
                        .collect(),
                    current,
                });
            }
        }
    }

    fn finish(mut self) -> Reconciliation {
        let mut taken: HashSet<DeviceRef> = self
            .known
            .iter()
            .map(|device| device.reference.clone())
            .collect();
        for (fingerprint, current) in group(&self.open_current, |index| Some(&self.current[index]))
        {
            if let [current] = current.as_slice() {
                let reference = mint_reference(fingerprint, &taken);
                taken.insert(reference.clone());
                self.result.new.push(IdentityMatch {
                    reference,
                    current: *current,
                });
            } else {
                self.result.ambiguous.push(AmbiguousGroup {
                    candidates: Vec::new(),
                    current,
                });
            }
        }
        self.result.missing = self
            .open_known
            .iter()
            .map(|&index| self.known[index].reference.clone())
            .collect();
        self.result.matched.sort_by_key(|matched| matched.current);
        self.result.ambiguous.sort_by_key(|group| group.current[0]);
        self.result
    }
}

fn group<K: PartialEq>(
    indices: &[usize],
    key: impl Fn(usize) -> Option<K>,
) -> Vec<(K, Vec<usize>)> {
    let mut groups: Vec<(K, Vec<usize>)> = Vec::new();
    for &index in indices {
        let Some(key) = key(index) else { continue };
        match groups.iter_mut().find(|(existing, _)| *existing == key) {
            Some((_, members)) => members.push(index),
            None => groups.push((key, vec![index])),
        }
    }
    groups
}

fn mint_reference(fingerprint: &DeviceFingerprint, taken: &HashSet<DeviceRef>) -> DeviceRef {
    let base = format!("dev-{:016x}", fnv1a(canonical(fingerprint).as_bytes()));
    let mut candidate = DeviceRef::new(base.clone());
    let mut suffix = 2;
    while taken.contains(&candidate) {
        candidate = DeviceRef::new(format!("{base}-{suffix}"));
        suffix += 1;
    }
    candidate
}

fn canonical(fingerprint: &DeviceFingerprint) -> String {
    let mut fields = vec![
        format!("{:?}", fingerprint.kind),
        fingerprint.vendor.clone(),
        fingerprint.model.clone(),
        fingerprint.serial.clone().unwrap_or_default(),
        fingerprint.location.clone().unwrap_or_default(),
    ];
    for zone in &fingerprint.zones {
        fields.push(format!(
            "{}:{:?}:{}:{}",
            zone.name, zone.kind, zone.leds_min, zone.leds_max
        ));
    }
    fields
        .iter()
        .map(|field| format!("{}:{field}", field.len()))
        .collect()
}

fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
