//! Commands that can change which profile applies at start publish the lifecycle view after they run.

use std::path::PathBuf;

use prismforge_core::RgbBackend;
use prismforge_core::automation::{RuleAction, Trigger};
use prismforge_core::profile::ProfileId;
use prismforge_core::store::ImportReport;

use crate::automation::{Automation, AutomationView};
use crate::lifecycle::{Lifecycle, LifecycleSink};
use crate::publisher::ViewSink;
use crate::shell::Shell;
use crate::view::{CommandError, ShellView};

pub async fn start<S: ViewSink, L: LifecycleSink, B: RgbBackend + Send + 'static>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    backend: B,
) -> Result<ShellView, CommandError> {
    let started = shell.start(backend).await;
    lifecycle.publish(shell).await;
    started
}

pub async fn stop<S: ViewSink, L: LifecycleSink>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
) -> ShellView {
    let view = shell.stop().await;
    lifecycle.publish(shell).await;
    view
}

pub async fn set_rule<S: ViewSink, L: LifecycleSink>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    automation: &Automation,
    trigger: Trigger,
    enabled: bool,
    action: RuleAction,
) -> Result<AutomationView, CommandError> {
    let changed = automation.set_rule(shell, trigger, enabled, action).await;
    lifecycle.publish(shell).await;
    changed
}

pub async fn rename_profile<S: ViewSink, L: LifecycleSink>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    id: &ProfileId,
    name: &str,
) -> Result<(), CommandError> {
    let renamed = shell.rename_profile(id, name).await;
    lifecycle.publish(shell).await;
    renamed
}

pub async fn delete_profile<S: ViewSink, L: LifecycleSink>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    id: &ProfileId,
) -> Result<(), CommandError> {
    let deleted = shell.delete_profile(id).await;
    lifecycle.publish(shell).await;
    deleted
}

pub async fn import_profile<S: ViewSink, L: LifecycleSink>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    source: PathBuf,
) -> Result<ImportReport, CommandError> {
    let imported = shell.import_profile(source).await;
    lifecycle.publish(shell).await;
    imported
}

pub async fn connect_openrgb<S: ViewSink, L: LifecycleSink, B: RgbBackend + Send + 'static>(
    shell: &Shell<S>,
    lifecycle: &Lifecycle<L>,
    host: &str,
    port: u16,
    backend: B,
) -> Result<ShellView, CommandError> {
    let _ = lifecycle.remember_address(shell, host, port).await;
    start(shell, lifecycle, backend).await
}
