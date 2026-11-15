//! Optional sampling runs in the check supervisor: no sampler child or thread can outlive it.

pub mod summary;

use anyhow::Result;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Serialize)]
pub(super) struct Report {
    file: &'static str,
    samples: u64,
    finished_unix_ms: Option<u128>,
    pub error: Option<String>,
}

pub(super) struct Capture {
    file: Option<File>,
    pub report: Report,
}

#[derive(Serialize)]
struct Reading {
    text: Option<String>,
    error: Option<String>,
}

impl Reading {
    fn read(path: impl AsRef<Path>) -> Self {
        match fs::read_to_string(path) {
            Ok(text) => Self {
                text: Some(text),
                error: None,
            },
            Err(error) => Self {
                text: None,
                error: Some(error.to_string()),
            },
        }
    }
}

#[derive(Serialize)]
struct Cgroup {
    path: PathBuf,
    readings: BTreeMap<&'static str, Reading>,
}

#[derive(Serialize)]
struct Sample {
    unix_ms: u128,
    host: BTreeMap<&'static str, Reading>,
    child_cgroup_source: Option<Reading>,
    child_cgroup: Option<Cgroup>,
}

/// Readings that [`summary`] reads back, named once for both sides.
const HOST_CPU: &str = "/proc/stat";
const HOST_CPU_PRESSURE: &str = "/proc/pressure/cpu";
const CGROUP_CPU: &str = "cpu.stat";

impl Sample {
    fn read(pid: Option<u32>) -> Result<Self> {
        let mut host: BTreeMap<_, _> = [
            "/proc/meminfo",
            HOST_CPU,
            HOST_CPU_PRESSURE,
            "/proc/pressure/memory",
            "/proc/pressure/io",
        ]
        .into_iter()
        .map(|path| (path, Reading::read(path)))
        .collect();
        // Aggregate CPU counters suffice; per-CPU/interrupt lists add noise and size.
        if let Some(text) = &mut host.get_mut(HOST_CPU).unwrap().text {
            *text = text.lines().next().unwrap_or_default().to_string();
        }
        let child_cgroup_source = pid.map(|pid| Reading::read(format!("/proc/{pid}/cgroup")));
        let child_cgroup = child_cgroup_source
            .as_ref()
            .and_then(|source| source.text.as_ref())
            .and_then(|text| text.lines().find_map(|line| line.strip_prefix("0::")))
            .map(|path| {
                let path = Path::new("/sys/fs/cgroup").join(path.trim_start_matches('/'));
                let readings = [
                    "memory.current",
                    "memory.peak",
                    "memory.swap.current",
                    "memory.events",
                    "memory.high",
                    "memory.max",
                    "memory.swap.max",
                    "cpu.max",
                    CGROUP_CPU,
                    "memory.pressure",
                    "io.pressure",
                ]
                .into_iter()
                .map(|name| (name, Reading::read(path.join(name))))
                .collect();
                Cgroup { path, readings }
            });
        Ok(Self {
            unix_ms: super::now()?,
            host,
            child_cgroup_source,
            child_cgroup,
        })
    }
}

impl Capture {
    pub fn start(directory: &Path) -> Self {
        let mut capture = Self {
            file: None,
            report: Report {
                file: "resources.jsonl",
                samples: 0,
                finished_unix_ms: None,
                error: None,
            },
        };
        match File::create_new(directory.join(capture.report.file)) {
            Ok(file) => capture.file = Some(file),
            Err(error) => capture.report.error = Some(error.to_string()),
        }
        capture.sample(None);
        capture
    }

    pub fn sample(&mut self, pid: Option<u32>) {
        if self.report.error.is_some() {
            return;
        }
        if let Err(error) = self.write_sample(pid) {
            self.report.error = Some(format!("{error:#}"));
        }
    }

    fn write_sample(&mut self, pid: Option<u32>) -> Result<()> {
        // Assemble the complete line before writing; a killed supervisor may still
        // leave a partial last line, which never constitutes completed evidence.
        let mut line = serde_json::to_vec(&Sample::read(pid)?)?;
        line.push(b'\n');
        self.file.as_mut().unwrap().write_all(&line)?;
        self.report.samples += 1;
        Ok(())
    }

    pub fn finish(&mut self) {
        self.sample(None);
        if let Some(file) = &self.file
            && let Err(error) = file.sync_all()
        {
            self.report.error = Some(error.to_string());
        }
        match super::now() {
            Ok(time) => self.report.finished_unix_ms = Some(time),
            Err(error) => self.report.error = Some(error.to_string()),
        }
    }
}
