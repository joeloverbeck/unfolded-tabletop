//! Locked file-replacement lifecycle. Adapters decide record
//! compatibility and replacement policy; this module owns filesystem sequencing.

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

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

fn check_size<E>(bytes: &[u8], limit: u64, error: fn(String) -> E) -> Result<(), E> {
    if bytes.len() as u64 > limit {
        return Err(error(
            "This file exceeds the bounded experiment's record limit.".into(),
        ));
    }
    Ok(())
}

/// Read at most one byte beyond the caller's limit, refusing oversized records.
pub fn read<E: From<std::io::Error>>(
    path: &Path,
    limit: u64,
    error: fn(String) -> E,
) -> Result<Vec<u8>, E> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    check_size(&bytes, limit, error)?;
    Ok(bytes)
}

/// Admission and any recovery validation run while the persistent sidecar is
/// locked. Original bytes, the lock, and every staged file stay owned here.
///
/// Returns the retained archive path only for an archived existing record.
/// Staged files are synced, but parent directories are not. This is not a
/// multi-file transaction: later failure can leave an archive or rotated recovery.
pub fn replace<E: From<std::io::Error>>(
    path: &Path,
    bytes: &[u8],
    limit: u64,
    error: fn(String) -> E,
    admit: impl FnOnce(Option<&[u8]>) -> Result<Replacement, E>,
    validate_recovery: impl FnOnce(&Path) -> Result<(), E>,
) -> Result<Option<PathBuf>, E> {
    check_size(bytes, limit, error)?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    // Never truncate or remove the sidecar: cooperating writers need its inode.
    let mut lock_path = path.as_os_str().to_os_string();
    lock_path.push(".lock");
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(PathBuf::from(lock_path))?;
    lock.try_lock()
        .map_err(|cause| error(format!("The save is busy: {cause}")))?;
    let existing = match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_file() => Some(read(path, limit, error)?),
        Ok(_) => {
            return Err(error(
                "The record destination is not a regular file.".into(),
            ));
        }
        Err(cause) if cause.kind() == std::io::ErrorKind::NotFound => None,
        Err(cause) => return Err(cause.into()),
    };
    let replacement = admit(existing.as_deref())?;
    if matches!(replacement, Replacement::Unchanged) {
        return Ok(None);
    }
    let previous = previous_path(path);
    if existing.is_some() && previous.try_exists()? {
        validate_recovery(&previous)?;
    }
    // Preserve the existing partial effects: this is not a multi-file
    // transaction. A later failure may leave an archive or rotated recovery.
    let mut next = tempfile::NamedTempFile::new_in(parent)?;
    next.write_all(bytes)?;
    next.as_file().sync_all()?;
    let mut archived_path = None;
    if let Some(existing) = existing {
        let mut backup = tempfile::NamedTempFile::new_in(parent)?;
        backup.write_all(&existing)?;
        backup.as_file().sync_all()?;
        if matches!(replacement, Replacement::Archive) {
            let prefix = format!(
                "{}.archived-",
                path.file_name().unwrap_or_default().to_string_lossy()
            );
            let mut archive = tempfile::Builder::new()
                .prefix(&prefix)
                .suffix(".json")
                .tempfile_in(parent)?;
            archive.write_all(&existing)?;
            archive.as_file().sync_all()?;
            let (_, path) = archive.keep().map_err(|cause| E::from(cause.error))?;
            archived_path = Some(path);
        }
        backup
            .persist(&previous)
            .map_err(|cause| E::from(cause.error))?;
    }
    if matches!(replacement, Replacement::Create) {
        next.persist_noclobber(path)
            .map_err(|cause| E::from(cause.error))?;
    } else {
        next.persist(path).map_err(|cause| E::from(cause.error))?;
    }
    Ok(archived_path)
}
