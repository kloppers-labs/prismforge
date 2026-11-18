fn main() {
    let manifest = tauri_build::AppManifest::new().commands(&[
        "view",
        "connect_openrgb",
        "start_demo",
        "disconnect",
        "refresh",
        "identify",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri build configuration");
}
