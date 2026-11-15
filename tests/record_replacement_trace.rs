//! Stop a real replacement at each rename's entry and exit to inspect lock
//! custody across both installations. No syscall is replaced. Linux ABI:
//! <linux/ptrace.h> and ptrace(2), https://man7.org/linux/man-pages/man2/ptrace.2.html.
//! Re-expresses Arkham's driver trace at the shared interface; it runs alone in
//! its own test executable because it waits for any traced thread.
#![cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]

use std::{
    collections::HashMap,
    ffi::{c_int, c_long, c_void},
    fs,
    os::unix::{fs::MetadataExt, process::CommandExt},
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use unfolded_tabletop::record_replacement::{Replacement, previous_path, replace};

const TRACED: &str = "UNFOLDED_TRACED_RECORD";

unsafe extern "C" {
    fn ptrace(request: c_int, pid: c_int, addr: *mut c_void, data: *mut c_void) -> c_long;
    fn waitpid(pid: c_int, status: *mut c_int, options: c_int) -> c_int;
}

#[repr(C)]
#[derive(Default)]
struct SyscallInfo {
    op: u8,
    reserved: u8,
    flags: u16,
    arch: u32,
    instruction_pointer: u64,
    stack_pointer: u64,
    // Enough aligned storage for the largest ptrace_syscall_info union member.
    data: [u64; 8],
}

struct Tracee(Option<Child>);

impl Drop for Tracee {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn request(operation: c_int, pid: c_int, address: usize, data: usize) -> c_long {
    // SAFETY: calls below supply either integer-valued options or a live,
    // correctly sized SyscallInfo buffer. The PID is a stopped traced thread.
    let result = unsafe { ptrace(operation, pid, address as *mut c_void, data as *mut c_void) };
    assert!(
        result >= 0,
        "ptrace {operation:#x}: {}",
        std::io::Error::last_os_error()
    );
    result
}

/// Wait for any traced thread. __WALL reports non-leader threads too; this
/// executable has no other children to reap.
fn wait_any(deadline: Instant) -> (c_int, c_int) {
    loop {
        let mut status = 0;
        // SAFETY: status is writable; WNOHANG bounds every poll.
        let result = unsafe { waitpid(-1, &mut status, 1 | 0x4000_0000) };
        if result > 0 {
            return (result, status);
        }
        assert!(result == 0, "waitpid: {}", std::io::Error::last_os_error());
        assert!(Instant::now() < deadline, "tracing exceeded 20 seconds");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn traced_replacement(path: &Path, mut at_install: impl FnMut(usize, bool)) {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "traced_replacement_child", "--test-threads=1"])
        .env(TRACED, path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit());
    // SAFETY: the child performs only an async-signal-safe ptrace call before
    // exec, without accessing parent state or taking any allocation locks.
    unsafe {
        command.pre_exec(|| {
            if ptrace(0, 0, std::ptr::null_mut(), std::ptr::null_mut()) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = Tracee(Some(command.spawn().expect(
        "start the test's own child with ptrace enabled; if child tracing is denied, \
         rerun the unchanged check in an environment that permits it",
    )));
    let mut child = child;
    let pid = child.0.as_ref().unwrap().id() as c_int;
    let deadline = Instant::now() + Duration::from_secs(20);
    assert_eq!(
        wait_any(deadline),
        (pid, 0x57f),
        "expected the initial exec SIGTRAP"
    );
    // PTRACE_O_TRACESYSGOOD | PTRACE_O_TRACECLONE | PTRACE_O_EXITKILL: the
    // test harness replaces in a thread of its own.
    request(0x4200, pid, 0, 1 | 8 | (1 << 20));
    request(24, pid, 0, 0); // PTRACE_SYSCALL
    let mut installing: HashMap<c_int, bool> = HashMap::from([(pid, false)]);
    let mut installs = 0;
    loop {
        let (tid, status) = wait_any(deadline);
        if status & 0x7f == 0 {
            installing.remove(&tid);
            if tid == pid {
                child.0.take(); // waitpid already reaped this exact child.
                assert_eq!(status >> 8, 0, "traced replacement exit status");
                break;
            }
            continue;
        }
        assert_eq!(
            status & 0xff,
            0x7f,
            "traced thread {tid} ended: {status:#x}"
        );
        let mut signal = 0;
        match status >> 8 {
            0x85 => {
                let mut info = SyscallInfo::default();
                request(
                    0x420e,
                    tid,
                    size_of::<SyscallInfo>(),
                    &mut info as *mut _ as usize,
                );
                let state = installing.entry(tid).or_default();
                if info.op == 1 {
                    // Linux x86_64 rename, renameat, renameat2; aarch64 has no rename.
                    *state = match info.arch {
                        0xc000003e => matches!(info.data[0], 82 | 264 | 316),
                        0xc00000b7 => matches!(info.data[0], 38 | 276),
                        arch => panic!("unsupported audit architecture {arch:#x}"),
                    };
                    if *state {
                        installs += 1;
                        at_install(installs, false);
                    }
                } else if info.op == 2 && *state {
                    assert_eq!(info.data[0], 0, "rename must succeed");
                    *state = false;
                    at_install(installs, true);
                }
            }
            // PTRACE_EVENT_CLONE; the new thread reports its own initial stop.
            0x305 => {}
            // A new thread's initial SIGSTOP is suppressed; other signals pass.
            0x13 if !installing.contains_key(&tid) => {
                installing.insert(tid, false);
            }
            other => signal = (other & 0x7f) as usize,
        }
        request(24, tid, 0, signal);
    }
    assert_eq!(installs, 2, "install recovery and then the active record");
}

#[test]
fn replacement_holds_the_same_sidecar_through_both_installations() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("record.json");
    let lock_path = directory.path().join("record.json.lock");
    replace(
        &path,
        b"original",
        1024,
        std::io::Error::other,
        |_| Ok(Replacement::Create),
        |_| Ok(()),
    )
    .unwrap();
    let contender = fs::File::open(&lock_path).unwrap();
    let inode = contender.metadata().unwrap().ino();
    traced_replacement(&path, |install, finished| {
        assert_eq!(fs::metadata(&lock_path).unwrap().ino(), inode);
        assert!(
            matches!(contender.try_lock(), Err(fs::TryLockError::WouldBlock)),
            "lock must cover install {install}, finished={finished}"
        );
        if install == 1 && !finished {
            assert!(!previous_path(&path).exists());
        } else {
            assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
        }
        if install == 1 || !finished {
            assert_eq!(fs::read(&path).unwrap(), b"original");
        }
    });
    assert_eq!(fs::read(&path).unwrap(), b"candidate");
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"original");
    contender.try_lock().unwrap();
}

/// The traced child: one replacement of the parent's record, then exit.
#[test]
fn traced_replacement_child() {
    let Some(path) = std::env::var_os(TRACED) else {
        return;
    };
    replace(
        Path::new(&path),
        b"candidate",
        1024,
        std::io::Error::other,
        |current| {
            assert_eq!(current, Some(b"original".as_slice()));
            Ok(Replacement::Replace)
        },
        |_| Ok(()),
    )
    .unwrap();
}
