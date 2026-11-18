use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::store::envelope::FileProblem;
use crate::store::files::FileOps;
use crate::store::notice::NoticeEvent;

/// Largest store file PrismForge reads; a larger file is treated as damaged.
pub const STORE_FILE_LIMIT: u64 = 4 * 1024 * 1024;

const TEMP: &str = ".tmp";
const BACKUP: &str = ".bak";
const BACKUP_TEMP: &str = ".bak.tmp";
const ASIDE: &str = ".unreadable-";
const ASIDE_ATTEMPTS: u32 = 1000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum WriteStep {
    Temporary,
    Sync,
    Backup,
    Replace,
}

/// A store file that was not written; the previous file is still in place.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[error("writing {file} stopped at step {step:?}: {detail}")]
pub struct WriteFailure {
    pub file: String,
    pub step: WriteStep,
    pub detail: String,
}

/// A file in the store directory; `name` is its path relative to that directory, with `/`.
#[derive(Debug, Clone)]
pub(crate) struct StoreFile {
    pub path: PathBuf,
    pub name: String,
}

pub(crate) enum Loaded<T> {
    Absent,
    Ready(T),
    Recovered(T, NoticeEvent),
    Empty(NoticeEvent),
    Untouched(FileProblem),
}

impl StoreFile {
    pub fn new(root: &Path, folder: Option<&str>, file_name: &str) -> Self {
        let directory = folder.map_or_else(|| root.to_path_buf(), |folder| root.join(folder));
        Self {
            path: directory.join(file_name),
            name: folder.map_or_else(
                || file_name.to_owned(),
                |folder| format!("{folder}/{file_name}"),
            ),
        }
    }

    fn sibling(&self, suffix: &str) -> PathBuf {
        let mut name = self
            .path
            .file_name()
            .expect("store files have a name")
            .to_os_string();
        name.push(suffix);
        self.path.with_file_name(name)
    }

    fn failure(&self, step: WriteStep, error: &io::Error) -> WriteFailure {
        WriteFailure {
            file: self.name.clone(),
            step,
            detail: error.to_string(),
        }
    }
}

/// Writes a temporary file, syncs it, keeps the current file as backup if `valid`, then replaces it.
pub(crate) fn write(
    ops: &dyn FileOps,
    file: &StoreFile,
    bytes: &[u8],
    valid: &dyn Fn(&[u8]) -> bool,
) -> Result<(), WriteFailure> {
    let temp = file.sibling(TEMP);
    put(ops, &temp, bytes).map_err(|(step, error)| file.failure(step, &error))?;
    if let Err(error) = back_up(ops, file, valid) {
        let _ = ops.remove(&temp);
        return Err(file.failure(WriteStep::Backup, &error));
    }
    if let Err(error) = ops.replace(&temp, &file.path) {
        let _ = ops.remove(&temp);
        return Err(file.failure(WriteStep::Replace, &error));
    }
    Ok(())
}

/// Removes the backup first, so an interrupted removal never brings the file back from it.
pub(crate) fn remove(ops: &dyn FileOps, file: &StoreFile) -> io::Result<()> {
    for path in [file.sibling(BACKUP), file.path.clone()] {
        match ops.remove(&path) {
            Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
            _ => {}
        }
    }
    Ok(())
}

/// Reads the file, or its backup when the file is missing or damaged; never reads past a newer version.
pub(crate) fn load<T>(
    ops: &dyn FileOps,
    file: &StoreFile,
    decode: &dyn Fn(&[u8]) -> Result<T, FileProblem>,
) -> Loaded<T> {
    let main_problem = match read(ops, &file.path, decode) {
        None => None,
        Some(Ok((value, _))) => return Loaded::Ready(value),
        Some(Err(problem @ FileProblem::NewerVersion { .. })) => {
            return Loaded::Untouched(problem);
        }
        Some(Err(problem)) => Some(problem),
    };
    let backup_path = file.sibling(BACKUP);
    match read(ops, &backup_path, decode) {
        None if main_problem.is_none() => Loaded::Absent,
        Some(Ok((value, bytes))) => {
            let set_aside = main_problem
                .is_some()
                .then(|| set_aside(ops, &file.path))
                .flatten();
            let main_cleared = main_problem.is_none() || set_aside.is_some();
            let rewritten = main_cleared && restore(ops, file, &bytes).is_ok();
            Loaded::Recovered(
                value,
                NoticeEvent::RestoredFromBackup {
                    problem: main_problem,
                    set_aside,
                    rewritten,
                },
            )
        }
        Some(Err(problem @ FileProblem::NewerVersion { .. })) => Loaded::Untouched(problem),
        backup => {
            let backup_problem = backup.and_then(Result::err);
            let mut aside = Vec::new();
            if main_problem.is_some() {
                aside.extend(set_aside(ops, &file.path));
            }
            if backup_problem.is_some() {
                aside.extend(set_aside(ops, &backup_path));
            }
            Loaded::Empty(NoticeEvent::StartedEmpty {
                main: main_problem,
                backup: backup_problem,
                set_aside: aside,
            })
        }
    }
}

pub(crate) fn main_of_backup(file_name: &str) -> Option<&str> {
    file_name.strip_suffix(BACKUP)
}

pub(crate) fn read_bounded(
    ops: &dyn FileOps,
    path: &Path,
    limit: u64,
) -> Result<Vec<u8>, FileProblem> {
    let bytes = ops
        .read(path, limit)
        .map_err(|error| FileProblem::Unreadable {
            detail: error.to_string(),
        })?;
    if bytes.len() as u64 > limit {
        return Err(FileProblem::TooLarge { limit });
    }
    Ok(bytes)
}

fn read<T>(
    ops: &dyn FileOps,
    path: &Path,
    decode: &dyn Fn(&[u8]) -> Result<T, FileProblem>,
) -> Option<Result<(T, Vec<u8>), FileProblem>> {
    if !ops.exists(path) {
        return None;
    }
    Some(read_bounded(ops, path, STORE_FILE_LIMIT).and_then(|bytes| Ok((decode(&bytes)?, bytes))))
}

fn put(ops: &dyn FileOps, temp: &Path, bytes: &[u8]) -> Result<(), (WriteStep, io::Error)> {
    let result = ops
        .write(temp, bytes)
        .map_err(|error| (WriteStep::Temporary, error))
        .and_then(|()| ops.sync(temp).map_err(|error| (WriteStep::Sync, error)));
    if result.is_err() {
        let _ = ops.remove(temp);
    }
    result
}

fn back_up(ops: &dyn FileOps, file: &StoreFile, valid: &dyn Fn(&[u8]) -> bool) -> io::Result<()> {
    if !ops.exists(&file.path) {
        return Ok(());
    }
    let Ok(current) = read_bounded(ops, &file.path, STORE_FILE_LIMIT) else {
        return Ok(());
    };
    if !valid(&current) {
        return Ok(());
    }
    let temp = file.sibling(BACKUP_TEMP);
    put(ops, &temp, &current).map_err(|(_, error)| error)?;
    ops.replace(&temp, &file.sibling(BACKUP)).inspect_err(|_| {
        let _ = ops.remove(&temp);
    })
}

fn restore(
    ops: &dyn FileOps,
    file: &StoreFile,
    bytes: &[u8],
) -> Result<(), (WriteStep, io::Error)> {
    let temp = file.sibling(TEMP);
    put(ops, &temp, bytes)?;
    ops.replace(&temp, &file.path).map_err(|error| {
        let _ = ops.remove(&temp);
        (WriteStep::Replace, error)
    })
}

/// Renames a damaged file next to itself under a free name; returns that name, or `None` if it stayed.
fn set_aside(ops: &dyn FileOps, path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    (1..=ASIDE_ATTEMPTS)
        .map(|attempt| format!("{name}{ASIDE}{attempt}"))
        .find(|candidate| !ops.exists(&path.with_file_name(candidate)))
        .filter(|candidate| ops.replace(path, &path.with_file_name(candidate)).is_ok())
}
