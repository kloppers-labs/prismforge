fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "view",
        "connect_openrgb",
        "start_demo",
        "disconnect",
        "refresh",
        "identify",
        "effect_catalog",
        "apply_effect",
        "stop_effect",
        "select_mode",
        "declare_external",
        "clear_external",
        "set_frame_rate",
        "preview_effect",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri build configuration");
}
