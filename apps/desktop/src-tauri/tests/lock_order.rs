mod support;

use std::future::Future;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use prismforge_core::profile::ProfileId;
use prismforge_core::{DeviceRef, OutputTarget};
use prismforge_desktop::{Shell, ShellConfig, Stores, stop_then_exit};
use prismforge_effects::EffectInstance;
use prismforge_testkit::DemoBackend;

use support::temp::TempDir;
use support::{Recorder, is_connected};

const PATIENCE: Duration = Duration::from_secs(600);

async fn ready(dir: &TempDir) -> (Shell<Recorder>, ProfileId) {
    let recorder = Recorder::default();
    let shell = Shell::new(
        recorder.clone(),
        ShellConfig::default(),
        Stores::new(dir.path().to_path_buf()),
    );
    shell.start(DemoBackend::new()).await.expect("start");
    recorder.wait_for(is_connected).await;
    shell.apply_effect(gpu(), rainbow()).await.expect("effect");
    let id = shell.create_profile("Glow").await.expect("save").id;
    (shell, id)
}

fn gpu() -> OutputTarget {
    OutputTarget {
        device: DeviceRef::new("demo-graphics-card"),
        zone: None,
    }
}

fn rainbow() -> EffectInstance {
    EffectInstance {
        effect: "rainbow-wave".into(),
        values: Default::default(),
    }
}

/// Fails instead of stalling the suite when the three calls wait on each other.
async fn finishes(label: &str, all: impl Future) {
    assert!(
        tokio::time::timeout(PATIENCE, all).await.is_ok(),
        "{label}: never finished"
    );
}

async fn still_responds(shell: &Shell<Recorder>) {
    finishes("a later disconnect", shell.stop()).await;
    finishes("a later start", shell.start(DemoBackend::new())).await;
}

#[tokio::test(start_paused = true)]
async fn apply_disconnect_and_a_frame_rate_change_all_finish() {
    let dir = TempDir::new("order-rate");
    let (shell, id) = ready(&dir).await;

    finishes("apply, stop, frame rate", async {
        tokio::join!(
            shell.apply_profile(&id),
            shell.stop(),
            shell.set_frame_rate(20)
        )
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn apply_disconnect_and_a_profile_save_all_finish() {
    let dir = TempDir::new("order-save");
    let (shell, id) = ready(&dir).await;

    finishes("apply, stop, save", async {
        tokio::join!(
            shell.apply_profile(&id),
            shell.stop(),
            shell.create_profile("During")
        )
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn apply_disconnect_and_identify_all_finish() {
    let dir = TempDir::new("order-identify");
    let (shell, id) = ready(&dir).await;

    finishes("apply, stop, identify", async {
        tokio::join!(
            shell.apply_profile(&id),
            shell.stop(),
            shell.identify(DeviceRef::new("demo-motherboard"), None)
        )
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn two_applies_and_a_disconnect_all_finish() {
    let dir = TempDir::new("order-apply");
    let (shell, id) = ready(&dir).await;

    finishes("apply, stop, apply", async {
        tokio::join!(
            shell.apply_profile(&id),
            shell.stop(),
            shell.apply_profile(&id)
        )
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn closing_the_window_during_an_apply_and_an_output_command_exits() {
    let dir = TempDir::new("order-exit");
    let (shell, id) = ready(&dir).await;
    let exited = AtomicBool::new(false);

    finishes("apply, window close, effect", async {
        tokio::join!(
            shell.apply_profile(&id),
            stop_then_exit(&shell, || exited.store(true, Ordering::SeqCst)),
            shell.apply_effect(gpu(), rainbow())
        )
    })
    .await;

    assert!(exited.load(Ordering::SeqCst));
}

async fn exported(shell: &Shell<Recorder>, id: &ProfileId, outside: &TempDir) -> PathBuf {
    let file = outside.path().join("glow.json");
    shell
        .export_profile(id, file.clone())
        .await
        .expect("export");
    file
}

#[tokio::test(start_paused = true)]
async fn an_import_and_a_disconnect_both_finish() {
    let dir = TempDir::new("order-import");
    let outside = TempDir::new("order-import-out");
    let (shell, id) = ready(&dir).await;
    let file = exported(&shell, &id, &outside).await;

    finishes("import, stop", async {
        tokio::join!(shell.import_profile(file), shell.stop())
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn an_apply_an_import_and_a_disconnect_all_finish() {
    let dir = TempDir::new("order-apply-import");
    let outside = TempDir::new("order-apply-import-out");
    let (shell, id) = ready(&dir).await;
    let file = exported(&shell, &id, &outside).await;

    finishes("apply, import, stop", async {
        tokio::join!(
            shell.apply_profile(&id),
            shell.import_profile(file),
            shell.stop()
        )
    })
    .await;
    still_responds(&shell).await;
}

#[tokio::test(start_paused = true)]
async fn closing_the_window_during_an_import_exits() {
    let dir = TempDir::new("order-import-exit");
    let outside = TempDir::new("order-import-exit-out");
    let (shell, id) = ready(&dir).await;
    let file = exported(&shell, &id, &outside).await;
    let exited = AtomicBool::new(false);

    finishes("import, window close", async {
        tokio::join!(
            shell.import_profile(file),
            stop_then_exit(&shell, || exited.store(true, Ordering::SeqCst))
        )
    })
    .await;

    assert!(exited.load(Ordering::SeqCst));
}
