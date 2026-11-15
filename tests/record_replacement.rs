//! File-replacement behavior retained from Arkham at 89be726, exercised through
//! the shared public boundary with test-owned admission and recovery validation.
//! Arkham's session admission and real driver tests stay there.

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
};
use unfolded_tabletop::record_replacement::{Replacement, previous_path, read, replace};

const LIMIT_ERROR: &str = "This file exceeds the bounded experiment's record limit.";

fn archives(path: &Path) -> Vec<PathBuf> {
    let prefix = format!("{}.archived-", path.file_name().unwrap().to_string_lossy());
    let mut paths: Vec<_> = fs::read_dir(path.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .collect();
    paths.sort();
    paths
}

fn lock_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".lock");
    name.into()
}

fn assert_unlocked(path: &Path) {
    fs::File::open(lock_path(path)).unwrap().try_lock().unwrap();
}

fn assert_no_staging_files(directory: &Path) {
    for entry in fs::read_dir(directory).unwrap() {
        let name = entry.unwrap().file_name();
        assert!(!name.to_string_lossy().starts_with(".tmp"), "{name:?}");
    }
}

/// Install `bytes` with a fixed decision, failing if recovery validation runs.
fn install(
    path: &Path,
    bytes: &[u8],
    decision: fn() -> Replacement,
) -> Result<Option<PathBuf>, Error> {
    replace(
        path,
        bytes,
        1024,
        Error::other,
        |_| Ok(decision()),
        |_| Ok(()),
    )
}

#[test]
fn exclusive_creation_preserves_a_file_that_appears_after_admission() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("attempt.json");
    let result = replace(
        &path,
        b"new attempt",
        1024,
        std::io::Error::other,
        |previous| {
            assert!(previous.is_none());
            fs::write(&path, b"competing attempt")?;
            Ok(Replacement::Create)
        },
        |_| Ok(()),
    );
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"competing attempt");
    assert!(!previous_path(&path).exists());
}

#[test]
fn oversized_candidate_is_refused_before_creating_directories_or_replacing_bytes() {
    let dir = tempfile::tempdir().unwrap();
    for path in [
        dir.path().join("missing/record"),
        dir.path().join("existing"),
    ] {
        if path.file_name().unwrap() == "existing" {
            fs::write(&path, b"keep").unwrap();
        }
        let result = replace(
            &path,
            b"too large",
            4,
            std::io::Error::other,
            |_| panic!("Oversized candidate reached admission"),
            |_| Ok(()),
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "This file exceeds the bounded experiment's record limit."
        );
    }
    assert!(!dir.path().join("missing").exists());
    assert_eq!(fs::read(dir.path().join("existing")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn continuation_and_restart_keep_original_bytes_and_durable_archives() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/record.json");
    install(&path, b"first", || Replacement::Create).unwrap();
    // Valid historical formatting must be copied, not regenerated.
    let original = b"first\n \t\n".to_vec();
    fs::write(&path, &original).unwrap();
    let seen = std::cell::RefCell::new(Vec::new());
    let archived = replace(
        &path,
        b"continued",
        1024,
        Error::other,
        |current| {
            seen.borrow_mut().push(current.map(<[u8]>::to_vec));
            Ok(Replacement::Replace)
        },
        |_| Ok(()),
    )
    .unwrap();
    assert!(archived.is_none());
    assert_eq!(seen.into_inner(), [Some(original.clone())]);
    assert_eq!(fs::read(&path).unwrap(), b"continued");
    assert_eq!(fs::read(previous_path(&path)).unwrap(), original);

    let archived = install(&path, b"restarted", || Replacement::Archive)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"restarted");
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"continued");
    let retained = archives(&path);
    assert_eq!(retained, std::slice::from_ref(&archived));
    assert!(archived.to_string_lossy().ends_with(".json"));
    assert_eq!(fs::read(&archived).unwrap(), b"continued");

    install(&path, b"after restart", || Replacement::Replace).unwrap();
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"restarted");
    assert_eq!(fs::read(&archived).unwrap(), b"continued");
    install(&path, b"second restart", || Replacement::Archive).unwrap();
    assert_eq!(archives(&path).len(), 2);
    assert_eq!(fs::read(&archived).unwrap(), b"continued");
    assert_unlocked(&path);
    assert_no_staging_files(path.parent().unwrap());
}

#[test]
fn unchanged_and_refused_decisions_return_before_recovery_validation() {
    for decision in ["unchanged", "refused"] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record.json");
        fs::write(&path, b"original\n \t\n").unwrap();
        fs::write(previous_path(&path), b"damaged recovery").unwrap();
        let result = replace(
            &path,
            b"candidate",
            1024,
            Error::other,
            |current| {
                assert_eq!(current, Some(b"original\n \t\n".as_slice()));
                if decision == "unchanged" {
                    Ok(Replacement::Unchanged)
                } else {
                    Err(Error::other("refused by admission"))
                }
            },
            |_| panic!("{decision} reached recovery validation"),
        );
        match decision {
            "unchanged" => assert!(result.unwrap().is_none()),
            _ => assert_eq!(result.unwrap_err().to_string(), "refused by admission"),
        }
        assert_eq!(fs::read(&path).unwrap(), b"original\n \t\n");
        assert_eq!(fs::read(previous_path(&path)).unwrap(), b"damaged recovery");
        assert!(archives(&path).is_empty());
        assert_unlocked(&path);
        assert_no_staging_files(dir.path());
    }
}

#[test]
fn damaged_recovery_blocks_replacement_but_is_ignored_for_a_fresh_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("record.json");
    fs::write(previous_path(&path), b"damaged recovery").unwrap();
    let validate = |previous: &Path| {
        assert_eq!(fs::read(previous).unwrap(), b"damaged recovery");
        assert!(
            fs::File::open(lock_path(&path))
                .unwrap()
                .try_lock()
                .is_err(),
            "recovery validation runs under the lock"
        );
        Err(Error::other("damaged recovery"))
    };
    // No current record: nothing rotates into recovery, so it is not validated.
    replace(
        &path,
        b"original",
        1024,
        Error::other,
        |current| {
            assert!(current.is_none());
            Ok(Replacement::Create)
        },
        |_| panic!("a fresh path validated recovery"),
    )
    .unwrap();
    for decision in [Replacement::Replace, Replacement::Archive] {
        let result = replace(
            &path,
            b"candidate",
            1024,
            Error::other,
            |_| Ok(decision),
            validate,
        );
        assert_eq!(result.unwrap_err().to_string(), "damaged recovery");
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read(previous_path(&path)).unwrap(), b"damaged recovery");
        assert!(archives(&path).is_empty());
        assert_unlocked(&path);
        assert_no_staging_files(dir.path());
    }
}

#[test]
fn admission_refusing_an_absent_record_creates_only_the_directory_and_sidecar() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/record.json");
    let result = replace(
        &path,
        b"candidate",
        1024,
        Error::other,
        |current| {
            assert!(current.is_none());
            Err(Error::other("missing record"))
        },
        |_| panic!("an absent record validated recovery"),
    );
    assert_eq!(result.unwrap_err().to_string(), "missing record");
    assert!(!path.exists());
    assert!(lock_path(&path).exists());
    assert!(!previous_path(&path).exists());
    assert!(archives(&path).is_empty());
    assert_unlocked(&path);
    assert_no_staging_files(path.parent().unwrap());
}

#[test]
fn non_regular_destinations_are_refused_and_preserved() {
    let dir = tempfile::tempdir().unwrap();
    let directory = dir.path().join("directory");
    fs::create_dir(&directory).unwrap();
    fs::write(directory.join("owned"), b"keep").unwrap();
    let mut destinations = vec![directory.clone()];
    #[cfg(unix)]
    {
        let link = dir.path().join("symlink");
        std::os::unix::fs::symlink(directory.join("owned"), &link).unwrap();
        destinations.push(link);
    }
    for path in destinations {
        let result = replace(
            &path,
            b"candidate",
            1024,
            Error::other,
            |_| panic!("a non-regular destination reached admission"),
            |_| panic!("a non-regular destination validated recovery"),
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "The record destination is not a regular file."
        );
        assert_eq!(fs::read(directory.join("owned")).unwrap(), b"keep");
        assert!(!previous_path(&path).exists());
    }
    #[cfg(unix)]
    assert!(
        fs::symlink_metadata(dir.path().join("symlink"))
            .unwrap()
            .is_symlink()
    );
}

#[test]
fn existing_record_limit_admits_exactly_the_limit_and_refuses_one_byte_more() {
    const LIMIT: usize = 64;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("record.json");
    let mut bytes = vec![b' '; LIMIT];
    fs::write(&path, &bytes).unwrap();
    assert_eq!(read(&path, LIMIT as u64, Error::other).unwrap(), bytes);
    let unchanged = replace(
        &path,
        &bytes,
        LIMIT as u64,
        Error::other,
        |current| {
            assert_eq!(current.map(<[u8]>::len), Some(LIMIT));
            Ok(Replacement::Unchanged)
        },
        |_| Ok(()),
    );
    assert!(unchanged.unwrap().is_none());
    bytes.push(b' ');
    fs::write(&path, &bytes).unwrap();
    assert_eq!(
        read(&path, LIMIT as u64, Error::other)
            .unwrap_err()
            .to_string(),
        LIMIT_ERROR
    );
    let refused = replace(
        &path,
        b"small",
        LIMIT as u64,
        Error::other,
        |_| panic!("an oversized existing record reached admission"),
        |_| Ok(()),
    );
    assert_eq!(refused.unwrap_err().to_string(), LIMIT_ERROR);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert!(!previous_path(&path).exists());
    assert_unlocked(&path);
}

#[test]
fn competing_lock_refuses_and_release_rechecks_the_current_record() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("record.json");
    fs::write(&path, b"original").unwrap();
    fs::write(previous_path(&path), b"original").unwrap();
    fs::write(lock_path(&path), b"persistent lock contents").unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(lock_path(&path))
        .unwrap();
    lock.try_lock().unwrap();
    let busy = replace(
        &path,
        b"candidate",
        1024,
        Error::other,
        |_| panic!("a busy record reached admission"),
        |_| panic!("a busy record validated recovery"),
    );
    assert!(
        busy.unwrap_err()
            .to_string()
            .starts_with("The save is busy: ")
    );
    assert_eq!(fs::read(&path).unwrap(), b"original");
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
    // A cooperating holder changes the active bytes before releasing the actual
    // sidecar. The next writer must read that version, not an earlier one.
    fs::write(&path, b"then current").unwrap();
    drop(lock);
    let refused = replace(
        &path,
        b"candidate",
        1024,
        Error::other,
        |current| {
            assert_eq!(current, Some(b"then current".as_slice()));
            Err(Error::other("a later record is already saved"))
        },
        |_| panic!("a refused decision validated recovery"),
    );
    assert_eq!(
        refused.unwrap_err().to_string(),
        "a later record is already saved"
    );
    assert_eq!(fs::read(&path).unwrap(), b"then current");
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
    assert_eq!(
        fs::read(lock_path(&path)).unwrap(),
        b"persistent lock contents"
    );
    assert_unlocked(&path);
}

#[test]
fn recovery_install_failure_cleans_staging_and_retains_only_a_restart_archive() {
    for decision in [Replacement::Replace, Replacement::Archive] {
        let restart = matches!(decision, Replacement::Archive);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record.json");
        let previous = previous_path(&path);
        fs::write(&path, b"original").unwrap();
        fs::write(&previous, b"valid recovery").unwrap();
        let result = replace(
            &path,
            b"candidate",
            1024,
            Error::other,
            |_| Ok(decision),
            |recovery| {
                assert_eq!(fs::read(recovery).unwrap(), b"valid recovery");
                // Validation passes; installation then meets a real directory.
                fs::remove_file(recovery).unwrap();
                fs::create_dir(recovery).unwrap();
                fs::write(recovery.join("owned"), b"keep").unwrap();
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(fs::read(&path).unwrap(), b"original");
        assert_eq!(fs::read(previous.join("owned")).unwrap(), b"keep");
        let retained = archives(&path);
        assert_eq!(retained.len(), usize::from(restart));
        if restart {
            assert_eq!(fs::read(&retained[0]).unwrap(), b"original");
        }
        assert_unlocked(&path);
        assert_no_staging_files(dir.path());
        fs::remove_dir_all(&previous).unwrap();
        install(&path, b"recovered", || Replacement::Replace).unwrap();
        assert_eq!(fs::read(&previous).unwrap(), b"original");
    }
}
