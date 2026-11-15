//! Process ownership and the digest at the crate's public interface (Linux `/proc`).

use std::{
    fs,
    os::unix::process::ExitStatusExt,
    path::Path,
    process::Command,
    thread,
    time::{Duration, Instant},
};
use unfolded_dev_tools::{
    digest,
    process::{self, Completion, Identity, OwnedChild},
};

fn alive(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{pid}/stat")).is_ok_and(|stat| {
        !stat
            .rsplit_once(')')
            .unwrap()
            .1
            .trim_start()
            .starts_with('Z')
    })
}

fn gone(pid: u32) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while alive(pid) {
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(20));
    }
    true
}

fn read_pid(path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(pid) = fs::read_to_string(path)
            .ok()
            .and_then(|text| text.trim().parse().ok())
        {
            return pid;
        }
        assert!(Instant::now() < deadline, "descendant pid was not written");
        thread::sleep(Duration::from_millis(10));
    }
}

/// A leader that leaves a sleeping descendant in its group and records its pid.
fn with_descendant(pid_file: &Path, leader: &str) -> Command {
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!("sleep 60 & echo $! > \"$1\"; {leader}"))
        .arg("sh")
        .arg(pid_file);
    command
}

#[test]
fn wait_reports_the_exit_status_or_a_timeout_and_cleanup_ends_a_running_child() {
    let mut exited = OwnedChild::spawn(Command::new("sh").args(["-c", "exit 7"])).unwrap();
    match exited.wait(Duration::from_secs(5)).unwrap() {
        Completion::Exited(status) => assert_eq!(status.code(), Some(7)),
        _ => panic!("the child exited"),
    }
    assert_eq!(exited.cleanup().unwrap().unwrap().code(), Some(7));

    let mut running = OwnedChild::spawn(Command::new("sleep").arg("60")).unwrap();
    let pid = running.id();
    assert!(matches!(
        running.wait(Duration::from_millis(50)).unwrap(),
        Completion::TimedOut
    ));
    assert!(alive(pid));
    let status = running.cleanup().unwrap().unwrap();
    assert_eq!(status.signal(), Some(libc::SIGTERM));
    assert!(!alive(pid));
    // A second cleanup reports the same reaped status without signalling again.
    assert_eq!(
        running.cleanup().unwrap().unwrap().signal(),
        Some(libc::SIGTERM)
    );
}

#[test]
fn cleanup_ends_descendants_left_in_the_group_after_the_leader_exits() {
    let directory = tempfile::tempdir().unwrap();
    let pid_file = directory.path().join("descendant.pid");
    let mut child = OwnedChild::spawn(&mut with_descendant(&pid_file, "exit 0")).unwrap();
    let descendant = read_pid(&pid_file);
    assert!(matches!(
        child.wait(Duration::from_secs(5)).unwrap(),
        Completion::Exited(status) if status.success()
    ));
    assert!(alive(descendant), "the descendant outlives its leader");
    assert!(child.cleanup().unwrap().unwrap().success());
    assert!(gone(descendant));
}

#[test]
fn dropping_an_owned_child_cleans_its_whole_group() {
    let directory = tempfile::tempdir().unwrap();
    let pid_file = directory.path().join("descendant.pid");
    let child = OwnedChild::spawn(&mut with_descendant(&pid_file, "wait")).unwrap();
    let leader = child.id();
    let descendant = read_pid(&pid_file);
    drop(child);
    assert!(!alive(leader));
    assert!(gone(descendant));
}

#[test]
fn a_foreground_child_keeps_the_callers_process_group() {
    let mut child = OwnedChild::spawn_foreground(Command::new("sleep").arg("60")).unwrap();
    let pid = child.id() as i32;
    // SAFETY: getpgid only reads process-group identity.
    assert_eq!(unsafe { libc::getpgid(pid) }, unsafe { libc::getpgid(0) });
    assert_eq!(
        child.cleanup().unwrap().unwrap().signal(),
        Some(libc::SIGTERM)
    );

    let mut grouped = OwnedChild::spawn(Command::new("sleep").arg("60")).unwrap();
    let pid = grouped.id() as i32;
    // SAFETY: as above.
    assert_eq!(unsafe { libc::getpgid(pid) }, pid);
    grouped.cleanup().unwrap();
}

#[test]
fn a_failing_wait_check_stops_the_wait_with_its_error() {
    let mut child = OwnedChild::spawn(Command::new("sleep").arg("60")).unwrap();
    let error = child
        .wait_checked(Duration::from_secs(5), || anyhow::bail!("caller refused"))
        .err()
        .unwrap();
    assert_eq!(error.to_string(), "caller refused");
    child.cleanup().unwrap();
}

#[test]
fn identity_binds_a_live_process_to_its_executable_file() {
    let sleep = fs::canonicalize(
        String::from_utf8(
            Command::new("sh")
                .args(["-c", "command -v sleep"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap()
        .trim(),
    )
    .unwrap();
    let mut child = OwnedChild::spawn(Command::new(&sleep).arg("60")).unwrap();
    let identity = Identity::read(child.id()).unwrap();
    assert_eq!(identity.executable, sleep);
    identity.verify(child.id()).unwrap();
    identity.verify_executable(&sleep).unwrap();
    let other = fs::canonicalize(std::env::current_exe().unwrap()).unwrap();
    assert!(
        identity
            .verify_executable(&other)
            .unwrap_err()
            .to_string()
            .contains("PID is not running the selected executable")
    );
    let pid = child.id();
    child.cleanup().unwrap();
    assert!(
        identity.verify(pid).is_err(),
        "an ended process has no identity"
    );
}

#[test]
fn pause_waits_the_whole_duration_without_an_interrupt() {
    assert!(!process::interrupted());
    process::check_interrupt().unwrap();
    let start = Instant::now();
    process::pause(Duration::from_millis(60)).unwrap();
    assert!(start.elapsed() >= Duration::from_millis(60));
}

#[test]
fn digest_is_the_sha256_of_the_file_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("abc");
    fs::write(&path, "abc").unwrap();
    assert_eq!(
        digest(&path).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let large = directory.path().join("large");
    fs::write(&large, vec![b'a'; 1_000_000]).unwrap();
    assert_eq!(
        digest(&large).unwrap(),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
    let missing = directory.path().join("missing");
    assert!(
        format!("{:#}", digest(&missing).unwrap_err())
            .contains(&format!("Cannot read {}", missing.display()))
    );
}
