//! A game's save record, and the locked file-replacement lifecycle under it.
//!
//! A game saves through [`save`]: that module owns the envelope, its checksum,
//! the write baseline, the unchanged shortcut, validation of the previous record
//! and recovery from it. The game supplies the identity that marks a compatible
//! save, its payload, its validity check and size limit, when a write is exclusive
//! or archives the current record, and the wording of every refusal.
//!
//! Underneath, [`replace`] and [`read`] own filesystem sequencing for any record,
//! save or not: bounded reads, a persistent sidecar lock ([`lock_path`]), staged
//! and synced replacement, previous-record retention ([`previous_path`]),
//! exclusive creation and restart archiving ([`archives`]). Their caller supplies
//! its record-size bound, replacement admission and recovery validation, and keeps
//! its own schema and freshness. Their refusals are causes in [`Error`], which the
//! caller words. Admission and recovery
//! validation run under the same lock as installation, and the lock file is never
//! truncated or removed. An unchanged decision writes no record bytes and does not
//! validate recovery.
//!
//! The lock is released explicitly when [`replace`] returns or unwinds, so a forked
//! child that has not yet reached exec cannot keep it held through an inherited
//! file description. Unlock errors, like close errors, are best-effort cleanup:
//! they do not mask the operation's error, report an installed record as failed,
//! or panic during unwinding. If the operating system refuses the unlock, an
//! inherited descriptor can still hold the lock until it closes.
//!
//! This is not a multi-file transaction, does not sync parent directories, does not
//! promise durability through power loss, and does not repair corrupt records.

pub mod save;

use std::{
    fmt, fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

struct ReplacementLock(fs::File);

impl Drop for ReplacementLock {
    fn drop(&mut self) {
        // Closing alone leaves the lock held by a child that inherited the open
        // file description and has not reached exec yet. Release at this scope's
        // end, including early returns and unwinding, before closing our file.
        // Like close errors, unlock errors must not replace an admission/write
        // error, turn an installed record into a reported failure, or panic.
        // Close remains the fallback if the OS refuses the explicit unlock.
        let _ = self.0.unlock();
    }
}

/// The caller's decision after inspecting the current bytes under the lock.
pub enum Replacement {
    /// Leave all record bytes unchanged.
    Unchanged,
    /// Install the candidate and retain the current record as its previous copy.
    Replace,
    /// Install only if the destination remains unused.
    Create,
    /// Also retain the current record in a separately named restart archive.
    Archive,
}

/// The adjacent recovery record, with `.previous` appended to the whole path.
pub fn previous_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".previous");
    PathBuf::from(name)
}

/// The persistent lock sidecar, with `.lock` appended to the whole path. It is
/// never truncated or removed.
pub fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".lock");
    PathBuf::from(name)
}

fn archive_prefix(path: &Path) -> String {
    format!(
        "{}.archived-",
        path.file_name().unwrap_or_default().to_string_lossy()
    )
}

/// The restart archives retained beside `path`, sorted by name: files named
/// `<file name>.archived-<random>.json`. An absent directory has none.
pub fn archives(path: &Path) -> std::io::Result<Vec<PathBuf>> {
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let prefix = archive_prefix(path);
    let entries = match fs::read_dir(parent) {
        Ok(entries) => entries,
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(cause) => return Err(cause),
    };
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) && name.ends_with(".json") {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}

/// Why [`read`] or [`replace`] refused or failed, as data a caller can match.
/// `Display` gives the default wording; a game that words a refusal itself
/// matches the cause instead of editing the text.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The record, or the candidate to be written, exceeds the caller's limit.
    TooLarge,
    /// Another writer holds the lock. The cause is the operating system's.
    Busy(std::io::Error),
    /// The destination is a link, directory or other non-regular file.
    NotRegularFile,
    /// The filesystem failed.
    Io(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge => f.write_str("The record exceeds its size limit."),
            Self::Busy(cause) => write!(f, "The record is busy: {cause}"),
            Self::NotRegularFile => f.write_str("The record destination is not a regular file."),
            Self::Io(cause) => cause.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Busy(cause) | Self::Io(cause) => Some(cause),
            Self::TooLarge | Self::NotRegularFile => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(cause: std::io::Error) -> Self {
        Self::Io(cause)
    }
}

/// A filesystem failure stays itself; every other cause becomes an error of kind
/// `Other` that carries it, so a caller whose own error is `std::io::Error` can
/// still recover the cause with `downcast`.
impl From<Error> for std::io::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::Io(cause) => cause,
            other => Self::other(other),
        }
    }
}

fn check_size(bytes: &[u8], limit: u64) -> Result<(), Error> {
    if bytes.len() as u64 > limit {
        return Err(Error::TooLarge);
    }
    Ok(())
}

/// Read at most one byte beyond `limit`, refusing an oversized record
/// ([`Error::TooLarge`]).
pub fn read(path: &Path, limit: u64) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    check_size(&bytes, limit)?;
    Ok(bytes)
}

/// Admission and any recovery validation run while the persistent sidecar is
/// locked. Original bytes, the lock, and every staged file stay owned here.
/// `admit` and `validate_recovery` return the caller's own error `E`; this
/// module's refusals and filesystem failures reach the caller through
/// `E: From<Error>`.
///
/// Returns the retained archive path only for an archived existing record.
/// Staged files are synced, but parent directories are not. This is not a
/// multi-file transaction: later failure can leave an archive or rotated recovery.
pub fn replace<E: From<Error>>(
    path: &Path,
    bytes: &[u8],
    limit: u64,
    admit: impl FnOnce(Option<&[u8]>) -> Result<Replacement, E>,
    validate_recovery: impl FnOnce(&Path) -> Result<(), E>,
) -> Result<Option<PathBuf>, E> {
    check_size(bytes, limit)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent).map_err(Error::Io)?;
    // Never truncate or remove the sidecar: cooperating writers need its inode.
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path(path))
        .map_err(Error::Io)?;
    lock.try_lock().map_err(|cause| {
        // Keep the lock's own wording, which reads differently from a bare
        // `WouldBlock`.
        let words = cause.to_string();
        Error::Busy(match cause {
            fs::TryLockError::Error(cause) => cause,
            fs::TryLockError::WouldBlock => {
                std::io::Error::new(std::io::ErrorKind::WouldBlock, words)
            }
        })
    })?;
    let _lock = ReplacementLock(lock);
    let existing = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Some(read(path, limit)?),
        Ok(_) => return Err(Error::NotRegularFile.into()),
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => None,
        Err(cause) => return Err(Error::Io(cause).into()),
    };
    let replacement = admit(existing.as_deref())?;
    if matches!(replacement, Replacement::Unchanged) {
        return Ok(None);
    }
    let previous = previous_path(path);
    if existing.is_some() && previous.try_exists().map_err(Error::Io)? {
        validate_recovery(&previous)?;
    }
    // Preserve the existing partial effects: this is not a multi-file
    // transaction. A later failure may leave an archive or rotated recovery.
    let mut next = tempfile::NamedTempFile::new_in(parent).map_err(Error::Io)?;
    next.write_all(bytes).map_err(Error::Io)?;
    next.as_file().sync_all().map_err(Error::Io)?;
    let mut archived_path = None;
    if let Some(existing) = existing {
        let mut backup = tempfile::NamedTempFile::new_in(parent).map_err(Error::Io)?;
        backup.write_all(&existing).map_err(Error::Io)?;
        backup.as_file().sync_all().map_err(Error::Io)?;
        if matches!(replacement, Replacement::Archive) {
            let mut archive = tempfile::Builder::new()
                .prefix(&archive_prefix(path))
                .suffix(".json")
                .tempfile_in(parent)
                .map_err(Error::Io)?;
            archive.write_all(&existing).map_err(Error::Io)?;
            archive.as_file().sync_all().map_err(Error::Io)?;
            let (_, path) = archive
                .keep()
                .map_err(|cause| E::from(Error::Io(cause.error)))?;
            archived_path = Some(path);
        }
        backup
            .persist(&previous)
            .map_err(|cause| E::from(Error::Io(cause.error)))?;
    }
    if matches!(replacement, Replacement::Create) {
        next.persist_noclobber(path)
            .map_err(|cause| E::from(Error::Io(cause.error)))?;
    } else {
        next.persist(path)
            .map_err(|cause| E::from(Error::Io(cause.error)))?;
    }
    Ok(archived_path)
}
