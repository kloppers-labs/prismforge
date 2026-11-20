use std::fs;
use std::path::{Path, PathBuf};

const LINT: &str = "unsafe_code";
const WIN32: &str = "crates/prismforge-system-events/src/windows.rs";
const SEAM_MANIFEST: &str = "crates/prismforge-system-events/Cargo.toml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).expect("folder") {
        let path = entry.expect("entry").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if name != "target" && name != "node_modules" && name != "gen" && !name.starts_with('.')
            {
                walk(&path, found);
            }
        } else if name.ends_with(".rs") || name == "Cargo.toml" {
            found.push(path);
        }
    }
}

fn relative(path: &Path) -> String {
    path.strip_prefix(root())
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn workspace_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for folder in ["crates", "apps/desktop/src-tauri"] {
        walk(&root().join(folder), &mut files);
    }
    files
}

fn squeezed(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn the_workspace_forbids_unsafe_code() {
    let manifest = squeezed(&fs::read_to_string(root().join("Cargo.toml")).expect("manifest"));
    assert!(manifest.contains(&format!("[workspace.lints.rust]{LINT}=\"forbid\"")));
}

#[test]
fn every_crate_but_the_event_seam_takes_the_workspace_lints_unchanged() {
    for manifest in workspace_files()
        .into_iter()
        .filter(|path| path.ends_with("Cargo.toml"))
    {
        let name = relative(&manifest);
        let text = squeezed(&fs::read_to_string(&manifest).expect("manifest"));
        if name == SEAM_MANIFEST {
            assert!(
                text.contains(&format!("[lints.rust]{LINT}=\"deny\"")),
                "{name}"
            );
            continue;
        }
        assert!(
            text.contains("[lints]workspace=true"),
            "{name} does not take the workspace lints"
        );
        assert!(!text.contains(LINT), "{name} changes {LINT}");
    }
}

#[test]
fn only_the_win32_module_names_the_unsafe_code_lint() {
    let naming: Vec<String> = workspace_files()
        .into_iter()
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .filter(|path| !path.ends_with("tests/unsafe_code.rs"))
        .filter(|path| fs::read_to_string(path).is_ok_and(|text| text.contains(LINT)))
        .map(|path| relative(&path))
        .collect();
    assert_eq!(naming, [WIN32]);
}
