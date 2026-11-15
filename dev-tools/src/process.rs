use anyhow::{Context, Result, bail};
use serde::Serialize;
use std::{
    fs,
    os::unix::{fs::MetadataExt, process::CommandExt},
    path::{Path, PathBuf},
    process::{Child, Command, ExitStatus},
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

pub fn install_interrupt_handler() -> Result<()> {
    ctrlc::set_handler(|| INTERRUPTED.store(true, Ordering::SeqCst))?;
    Ok(())
}

pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

pub fn check_interrupt() -> Result<()> {
    if interrupted() {
        bail!("interrupted; no complete result is claimed");
    }
    Ok(())
}

pub fn pause(duration: Duration) -> Result<()> {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        check_interrupt()?;
        thread::sleep(
            deadline
                .saturating_duration_since(Instant::now())
                .min(Duration::from_millis(20)),
        );
    }
    check_interrupt()
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Identity {
    pub start_ticks: String,
    pub executable: PathBuf,
    pub device: u64,
    pub inode: u64,
}

#[derive(Debug, PartialEq, Eq)]
struct Lifetime {
    start_ticks: String,
    group: String,
}

impl Lifetime {
    fn read(pid: u32) -> Result<Self> {
        let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
            .context("App process is absent or not visible here")?;
        // comm may contain spaces or ')'; field 22 follows the final ')'.
        let fields: Vec<_> = stat
            .rsplit_once(')')
            .context("Invalid process stat")?
            .1
            .split_whitespace()
            .collect();
        Ok(Self {
            start_ticks: fields
                .get(19)
                .context("Cannot read process start identity")?
                .to_string(),
            group: fields
                .get(2)
                .context("Cannot read process group identity")?
                .to_string(),
        })
    }
}

impl Identity {
    pub fn read(pid: u32) -> Result<Self> {
        let proc = PathBuf::from(format!("/proc/{pid}"));
        let start_ticks = Lifetime::read(pid)?.start_ticks;
        let executable = fs::read_link(proc.join("exe"))?;
        let file = fs::metadata(proc.join("exe"))?;
        Ok(Self {
            start_ticks,
            executable,
            device: file.dev(),
            inode: file.ino(),
        })
    }

    pub fn verify(&self, pid: u32) -> Result<()> {
        if Self::read(pid)? != *self {
            bail!("App process identity changed");
        }
        Ok(())
    }

    pub fn verify_executable(&self, expected: &Path) -> Result<()> {
        let file = fs::metadata(expected)?;
        if self.executable != expected || self.device != file.dev() || self.inode != file.ino() {
            bail!("PID is not running the selected executable; check the PID and build path");
        }
        Ok(())
    }
}

pub enum Completion {
    Exited(ExitStatus),
    TimedOut,
    Interrupted,
}

/// Owns one newly created process group, including descendants left by its leader.
pub struct OwnedChild {
    child: Child,
    lifetime: Option<Lifetime>,
    group: bool,
    cleaned: bool,
}

impl OwnedChild {
    pub fn spawn(command: &mut Command) -> Result<Self> {
        check_interrupt()?;
        let child = command.process_group(0).spawn()?;
        Self::own(child, true)
    }

    /// Authentication keeps the terminal's foreground group so sudo can read locally.
    pub fn spawn_foreground(command: &mut Command) -> Result<Self> {
        check_interrupt()?;
        let child = command.spawn()?;
        Self::own(child, false)
    }

    fn own(child: Child, group: bool) -> Result<Self> {
        // Until first wait the child cannot be reaped or its PID reused. Drop
        // cleans this newly owned group even if reading /proc unexpectedly fails.
        let mut owned = Self {
            child,
            lifetime: None,
            group,
            cleaned: false,
        };
        owned.lifetime = Some(Lifetime::read(owned.id())?);
        Ok(owned)
    }

    pub fn id(&self) -> u32 {
        self.child.id()
    }

    pub fn wait(&mut self, timeout: Duration) -> Result<Completion> {
        self.wait_checked(timeout, || Ok(()))
    }

    pub fn wait_checked(
        &mut self,
        timeout: Duration,
        check: impl Fn() -> Result<()>,
    ) -> Result<Completion> {
        let deadline = Instant::now() + timeout;
        loop {
            check()?;
            if interrupted() {
                return Ok(Completion::Interrupted);
            }
            if let Some(status) = self.child.try_wait()? {
                return Ok(Completion::Exited(status));
            }
            if Instant::now() >= deadline {
                return Ok(Completion::TimedOut);
            }
            thread::sleep(Duration::from_millis(10));
        }
    }

    pub fn cleanup(&mut self) -> Result<Option<ExitStatus>> {
        if self.cleaned {
            return self.child.try_wait().map_err(Into::into);
        }
        self.signal_group(libc::SIGTERM)?;
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && self.child.try_wait()?.is_none() {
            thread::sleep(Duration::from_millis(20));
        }
        // The leader may have exited while a descendant still owns the group.
        self.signal_group(libc::SIGKILL)?;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Some(status) = self.child.try_wait()? {
                self.cleaned = true;
                return Ok(Some(status));
            }
            if Instant::now() >= deadline {
                bail!("Owned process did not exit within cleanup deadline");
            }
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn signal_group(&self, signal: i32) -> Result<()> {
        // exec changes an executable's path/inode without changing ownership.
        // Recheck the lifetime before each signal, including after waiting.
        if let Some(original) = &self.lifetime {
            match Lifetime::read(self.id()) {
                Ok(current) if original != &current => {
                    bail!(
                        "Owned process lifetime changed; refusing to signal a reused PID or group"
                    );
                }
                Ok(_) => {}
                // A reaped leader may leave its descendants in the owned group.
                Err(error)
                    if error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| error.kind() == std::io::ErrorKind::NotFound) => {}
                Err(error) => {
                    return Err(error.context("Cannot revalidate owned process before cleanup"));
                }
            }
        }
        let target = if self.group {
            -(self.id() as i32)
        } else {
            self.id() as i32
        };
        // SAFETY: target is our spawned child or its newly created process group.
        let result = unsafe { libc::kill(target, signal) };
        if result != 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() != Some(libc::ESRCH) {
                return Err(error.into());
            }
        }
        Ok(())
    }
}

impl Drop for OwnedChild {
    fn drop(&mut self) {
        if !self.cleaned
            && let Err(error) = self.cleanup()
        {
            eprintln!("Owned process cleanup failed: {error:#}");
        }
    }
}
