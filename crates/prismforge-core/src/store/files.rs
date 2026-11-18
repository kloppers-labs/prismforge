use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

/// The file operations the store uses; tests replace them to fail at a chosen step.
pub trait FileOps: Send + Sync {
    /// Reads at most `limit + 1` bytes, so the caller can tell that a file is too large.
    fn read(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>>;

    /// Creates or truncates `path` and writes `bytes`; does not flush to disk.
    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()>;

    /// Flushes the contents of `path` to disk.
    fn sync(&self, path: &Path) -> io::Result<()>;

    /// Moves `from` over `to` in one step, replacing `to` if it exists.
    fn replace(&self, from: &Path, to: &Path) -> io::Result<()>;

    fn remove(&self, path: &Path) -> io::Result<()>;

    fn exists(&self, path: &Path) -> bool;

    /// The names of the files in `dir`; an absent directory has none.
    fn list(&self, dir: &Path) -> io::Result<Vec<String>>;

    fn create_dir_all(&self, dir: &Path) -> io::Result<()>;
}

/// The real file system, through the standard library only.
#[derive(Debug, Clone, Copy, Default)]
pub struct StdFileOps;

impl FileOps for StdFileOps {
    fn read(&self, path: &Path, limit: u64) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        File::open(path)?
            .take(limit.saturating_add(1))
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }

    fn write(&self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        File::create(path)?.write_all(bytes)
    }

    fn sync(&self, path: &Path) -> io::Result<()> {
        OpenOptions::new().write(true).open(path)?.sync_all()
    }

    fn replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)?;
        // A directory sync makes the rename durable; Windows cannot open a directory here, so it is skipped.
        if let Some(Ok(directory)) = to.parent().map(File::open) {
            let _ = directory.sync_all();
        }
        Ok(())
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn list(&self, dir: &Path) -> io::Result<Vec<String>> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_file()
                && let Some(name) = entry.file_name().to_str()
            {
                names.push(name.to_owned());
            }
        }
        names.sort();
        Ok(names)
    }

    fn create_dir_all(&self, dir: &Path) -> io::Result<()> {
        fs::create_dir_all(dir)
    }
}
