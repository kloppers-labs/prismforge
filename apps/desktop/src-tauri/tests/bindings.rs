#![cfg(feature = "ts")]

mod generated;

use prismforge_core::store::{ApplyOutcome, ExportReport, ImportReport, ProfileDetails, Saved};
use prismforge_desktop::{CommandError, EffectCatalog, ShellView, StoreView};
use prismforge_effects::{PreviewBatch, PreviewRequest};
use ts_rs::{Config, TS};

use generated::Generated;

#[test]
fn committed_bindings_match_the_rust_types() {
    let generated = Generated::new("src/lib/api/bindings");
    let config = Config::new()
        .with_large_int("number")
        .with_out_dir(generated.scratch());
    ShellView::export_all(&config).expect("export ShellView");
    CommandError::export_all(&config).expect("export CommandError");
    EffectCatalog::export_all(&config).expect("export EffectCatalog");
    PreviewRequest::export_all(&config).expect("export PreviewRequest");
    PreviewBatch::export_all(&config).expect("export PreviewBatch");
    StoreView::export_all(&config).expect("export StoreView");
    ProfileDetails::export_all(&config).expect("export ProfileDetails");
    Saved::export_all(&config).expect("export Saved");
    ApplyOutcome::export_all(&config).expect("export ApplyOutcome");
    ImportReport::export_all(&config).expect("export ImportReport");
    ExportReport::export_all(&config).expect("export ExportReport");

    generated.compare_or_update();
}
