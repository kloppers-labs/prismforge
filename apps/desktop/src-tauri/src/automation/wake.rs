use prismforge_core::DeviceRef;
use prismforge_core::automation::{
    RuleRun, RunFailure, Trigger, WakeReady, WakeTarget, wait_after_wake,
};
use prismforge_core::session::SessionHandle;
use tokio::time::timeout;

use crate::automation::run::{failed, is_live, publish, record, run_of, run_rule};
use crate::automation::{WAKE_APPLY_LIMIT, WAKE_STEP_LIMIT};
use crate::publisher::ViewSink;
use crate::shell::Shell;

/// Reaches the session through the shell for each step, so the wait never holds a session handle; each step is capped.
struct ShellTarget<'a, S: ViewSink> {
    shell: &'a Shell<S>,
}

impl<S: ViewSink> WakeTarget for ShellTarget<'_, S> {
    async fn is_live(&self) -> bool {
        timeout(WAKE_STEP_LIMIT, self.shell.snapshot())
            .await
            .ok()
            .flatten()
            .is_some_and(|session| is_live(&session.connection))
    }

    async fn reconnect(&self) {
        let reconnect = async {
            if let Ok(connect) = self.shell.queue(SessionHandle::connect).await {
                let _ = connect.await;
            }
        };
        let _ = timeout(WAKE_STEP_LIMIT, reconnect).await;
    }

    async fn devices(&self) -> Option<Vec<DeviceRef>> {
        let read = async {
            self.shell
                .queue(SessionHandle::refresh)
                .await
                .ok()?
                .await
                .ok()?;
            self.shell.snapshot().await
        };
        let snapshot = timeout(WAKE_STEP_LIMIT, read).await.ok().flatten()?;
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

/// The session's id, or why the wake rule cannot run on it.
async fn session_of<S: ViewSink>(shell: &Shell<S>) -> Result<SessionHandle, RunFailure> {
    match timeout(WAKE_STEP_LIMIT, shell.current()).await {
        Ok((Some(handle), _)) => Ok(handle),
        Ok((None, _)) => Err(RunFailure::NotConnected),
        Err(_) => Err(RunFailure::TimedOut {
            limit_ms: WAKE_STEP_LIMIT.as_millis() as u64,
        }),
    }
}

/// `current` is false once a newer sleep started; the wake rule then does not run.
pub(crate) async fn run_wake<S: ViewSink>(
    shell: &Shell<S>,
    before: &[(DeviceRef, String)],
    current: impl Fn() -> bool,
) -> Option<RuleRun> {
    let kind = shell
        .view()
        .session
        .map_or(crate::stores::StoreKind::Real, |session| {
            crate::stores::StoreKind::of(session.backend.origin)
        });
    let profiles = timeout(WAKE_STEP_LIMIT, shell.stores.profiles(kind))
        .await
        .ok()?
        .ok()?;
    timeout(WAKE_STEP_LIMIT, profiles.rules())
        .await
        .ok()?
        .rule(Trigger::Wakes)
        .due()?;
    let not_run = |failure| async {
        let run = run_of(Trigger::Wakes, failed(failure));
        let _ = timeout(WAKE_STEP_LIMIT, record(&profiles, run.clone())).await;
        Some(publish(&shell.publisher, run))
    };
    let session = match session_of(shell).await {
        Ok(handle) => handle.session_id(),
        Err(failure) => return not_run(failure).await,
    };
    let refs: Vec<DeviceRef> = before.iter().map(|(device, _)| device.clone()).collect();
    let ready = wait_after_wake(&ShellTarget { shell }, &refs).await;
    let missing = match ready {
        WakeReady::NeverBack { waited } => {
            let failure = RunFailure::OpenRgbNotBack {
                waited_ms: waited.as_millis() as u64,
            };
            return not_run(failure).await;
        }
        WakeReady::Ready { missing, .. } => before
            .iter()
            .filter(|(device, _)| missing.contains(device))
            .map(|(_, name)| name.clone())
            .collect(),
    };
    let handle = match session_of(shell).await {
        Ok(handle) if handle.session_id() == session => handle,
        Ok(_) => return None,
        Err(failure) => return not_run(failure).await,
    };
    if !current() {
        return None;
    }
    shell.checks.refresh(&handle).await;
    let _ = handle.resync().await;
    drop(handle);
    run_rule(shell, Trigger::Wakes, WAKE_APPLY_LIMIT, missing, &current).await
}
