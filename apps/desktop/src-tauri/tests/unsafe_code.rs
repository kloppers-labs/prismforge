use std::fs;
use std::path::{Path, PathBuf};

const ALLOWED: &str = "apps/desktop/src-tauri/src/system_events/windows.rs";

fn rust_files(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).expect("folder") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() && name != "target" && name != "node_modules" && !name.starts_with('.') {
            rust_files(&path, found);
        } else if name.ends_with(".rs") {
            found.push(path);
        }
    }
}

#[test]
fn only_the_win32_module_allows_unsafe_code() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut files = Vec::new();
    for folder in [
        "crates",
        "apps/desktop/src-tauri/src",
        "apps/desktop/src-tauri/tests",
    ] {
        rust_files(&root.join(folder), &mut files);
    }
    let marker = ["allow(", "unsafe_code)"].concat();
    let allowing: Vec<String> = files
        .iter()
        .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(&marker)))
        .map(|path| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(allowing, [ALLOWED]);
}
