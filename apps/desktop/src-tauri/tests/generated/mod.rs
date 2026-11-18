#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const UPDATE: &str = "PRISMFORGE_UPDATE_GENERATED";

/// A directory of generated frontend files that must equal what the Rust code produces now.
pub struct Generated {
    committed: PathBuf,
    scratch: PathBuf,
}

impl Generated {
    pub fn new(relative_to_app: &str) -> Self {
        let app = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let name = relative_to_app.replace('/', "-");
        let scratch =
            std::env::temp_dir().join(format!("prismforge-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&scratch);
        fs::create_dir_all(&scratch).expect("scratch directory");
        Self {
            committed: app.join(relative_to_app),
            scratch,
        }
    }

    pub fn scratch(&self) -> &Path {
        &self.scratch
    }

    pub fn write(&self, name: &str, content: &str) {
        fs::write(self.scratch.join(name), content).expect("write generated file");
    }

    pub fn compare_or_update(self) {
        let fresh = read_dir(&self.scratch);
        if std::env::var_os(UPDATE).is_some() {
            let _ = fs::remove_dir_all(&self.committed);
            fs::create_dir_all(&self.committed).expect("committed directory");
            for (name, content) in &fresh {
                fs::write(self.committed.join(name), content).expect("update committed file");
            }
        } else {
            let committed = read_dir(&self.committed);
            assert!(
                committed == fresh,
                "{} is out of date. Run `{UPDATE}=1 cargo test -p prismforge-desktop --all-features --test bindings --test fixtures` and commit the result.",
                self.committed.display()
            );
        }
        let _ = fs::remove_dir_all(&self.scratch);
    }
}

fn read_dir(directory: &Path) -> BTreeMap<String, String> {
    let Ok(entries) = fs::read_dir(directory) else {
        return BTreeMap::new();
    };
    entries
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.is_file())
        .map(|path| {
            let name = path
                .file_name()
                .expect("file name")
                .to_string_lossy()
                .into_owned();
            let content = fs::read_to_string(&path)
                .expect("read file")
                .replace("\r\n", "\n");
            (name, content)
        })
        .collect()
}
