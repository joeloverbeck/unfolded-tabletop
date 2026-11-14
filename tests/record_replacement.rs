//! File-replacement behavior retained from Arkham at 89be726, exercised through
//! the shared public boundary. Game admission and real driver tests stay there.

use std::fs;
use unfolded_tabletop::record_replacement::{Replacement, previous_path, replace};

#[cfg(target_os = "linux")]
mod inherited_lock {
    use super::*;
    use std::{
        io::{Read, Write},
        os::unix::{net::UnixStream, process::CommandExt},
        process::{Command, ExitStatus, Stdio},
        thread::{self, JoinHandle},
        time::Duration,
    };

    // Hold a real fork's inherited descriptors before exec closes CLOEXEC files.
    // The handshake fixes the ordering; timeouts only bound a broken test.
    struct BeforeExec {
        release: UnixStream,
        child: Option<JoinHandle<std::io::Result<ExitStatus>>>,
    }

    impl BeforeExec {
        fn hold() -> Self {
            let (release, child_socket) = UnixStream::pair().unwrap();
            release
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            child_socket
                .set_read_timeout(Some(Duration::from_secs(10)))
                .unwrap();
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .arg("--list")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            // SAFETY: after fork, only read/write syscalls and stack-local values
            // are used. No allocation, locks, or test assertions run in the child.
            unsafe {
                command.pre_exec(move || {
                    if rustix::io::write(&child_socket, b"r")? != 1
                        || rustix::io::read(&child_socket, &mut [0])? != 1
                    {
                        return Err(std::io::Error::from_raw_os_error(5));
                    }
                    Ok(())
                });
            }
            let mut held = Self {
                release,
                child: Some(thread::spawn(move || command.status())),
            };
            held.release.read_exact(&mut [0]).unwrap();
            held
        }
    }

    impl Drop for BeforeExec {
        fn drop(&mut self) {
            // Release and reap even when an assertion or the admission panics.
            let _ = self.release.write_all(b"g");
            let status = self.child.take().unwrap().join();
            if !thread::panicking() {
                assert!(status.unwrap().unwrap().success());
            }
        }
    }

    #[test]
    fn completed_replacement_releases_lock_while_child_is_still_before_exec() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record");
        let mut child = None;
        replace(
            &path,
            b"first",
            1024,
            std::io::Error::other,
            |_| {
                child = Some(BeforeExec::hold());
                Ok(Replacement::Replace)
            },
            |_| Ok(()),
        )
        .unwrap();
        replace(
            &path,
            b"second",
            1024,
            std::io::Error::other,
            |current| {
                assert_eq!(current, Some(b"first".as_slice()));
                Ok(Replacement::Replace)
            },
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        assert_eq!(fs::read(previous_path(&path)).unwrap(), b"first");
        drop(child);
    }

    #[test]
    fn unchanged_errors_and_unwind_release_lock_before_child_exec() {
        for exit in ["unchanged", "admission", "recovery", "install", "unwind"] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("record");
            fs::write(&path, b"original").unwrap();
            fs::write(previous_path(&path), b"older").unwrap();
            let mut child = None;
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                replace(
                    &path,
                    b"candidate",
                    1024,
                    std::io::Error::other,
                    |_| {
                        child = Some(BeforeExec::hold());
                        match exit {
                            "unchanged" => Ok(Replacement::Unchanged),
                            "admission" => Err(std::io::Error::other("admission refused")),
                            "install" => Ok(Replacement::Create),
                            "unwind" => panic!("admission panicked"),
                            _ => Ok(Replacement::Replace),
                        }
                    },
                    |_| {
                        assert_ne!(exit, "unchanged", "Unchanged must not validate recovery");
                        if exit == "recovery" {
                            Err(std::io::Error::other("recovery refused"))
                        } else {
                            Ok(())
                        }
                    },
                )
            }));
            match exit {
                "unchanged" => assert!(result.unwrap().unwrap().is_none()),
                "unwind" => assert!(result.is_err()),
                "admission" | "recovery" => assert_eq!(
                    result.unwrap().unwrap_err().to_string(),
                    format!("{exit} refused")
                ),
                "install" => assert_eq!(
                    result.unwrap().unwrap_err().kind(),
                    std::io::ErrorKind::AlreadyExists
                ),
                _ => unreachable!(),
            }
            assert_eq!(fs::read(&path).unwrap(), b"original", "{exit}");
            // Exclusive creation retains the existing partial recovery rotation.
            assert_eq!(
                fs::read(previous_path(&path)).unwrap(),
                if exit == "install" {
                    b"original".as_slice()
                } else {
                    b"older".as_slice()
                },
                "{exit}"
            );
            replace(
                &path,
                b"next",
                1024,
                std::io::Error::other,
                |current| {
                    assert_eq!(current, Some(b"original".as_slice()));
                    Ok(Replacement::Replace)
                },
                |_| Ok(()),
            )
            .unwrap_or_else(|cause| panic!("{exit} left the lock held: {cause}"));
            assert_eq!(fs::read(&path).unwrap(), b"next");
            assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
            drop(child);
        }
    }

    #[test]
    fn independent_writer_is_refused_through_admission_and_recovery_validation() {
        use std::os::unix::fs::MetadataExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("record");
        fs::write(&path, b"original").unwrap();
        fs::write(previous_path(&path), b"older").unwrap();
        let lock_path = dir.path().join("record.lock");
        fs::write(&lock_path, b"stable sidecar").unwrap();
        let inode = fs::metadata(&lock_path).unwrap().ino();
        let contend = || {
            let output = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "inherited_lock::independent_writer_child"])
                .env("UNFOLDED_RECORD_CONTENDER", &path)
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        };
        replace(
            &path,
            b"installed",
            1024,
            std::io::Error::other,
            |current| {
                assert_eq!(current, Some(b"original".as_slice()));
                contend();
                Ok(Replacement::Replace)
            },
            |previous| {
                assert_eq!(fs::read(previous).unwrap(), b"older");
                contend();
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"installed");
        assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
        assert_eq!(fs::metadata(&lock_path).unwrap().ino(), inode);
        assert_eq!(fs::read(lock_path).unwrap(), b"stable sidecar");
    }

    #[test]
    fn independent_writer_child() {
        let Some(path) = std::env::var_os("UNFOLDED_RECORD_CONTENDER") else {
            return;
        };
        let path = std::path::Path::new(&path);
        let result = replace(
            path,
            b"competing",
            1024,
            std::io::Error::other,
            |_| panic!("Competing writer reached admission"),
            |_| panic!("Competing writer reached recovery validation"),
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .starts_with("The save is busy:")
        );
        assert_eq!(fs::read(path).unwrap(), b"original");
        assert_eq!(fs::read(previous_path(path)).unwrap(), b"older");
    }
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
