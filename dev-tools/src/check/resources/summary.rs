//! Read a recorder's `resources.jsonl` back as CPU use over time windows: the
//! host's busy share, the recorded command's cgroup in cores, and the share of
//! time some task waited for a CPU. Every figure is a counter difference between
//! two consecutive samples, shared between windows by time.

use super::{CGROUP_CPU, HOST_CPU, HOST_CPU_PRESSURE};
use anyhow::{Context, Result, bail, ensure};
use clap::Args as ClapArgs;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    io::Write,
    path::PathBuf,
};

#[derive(ClapArgs)]
pub struct Args {
    /// A `record-check --resources` output directory, holding resources.jsonl.
    output_dir: PathBuf,
    /// Unix time in milliseconds that window times count from, such as a frame
    /// record's start; by default the first sample's time.
    #[arg(long)]
    origin_unix_ms: Option<u64>,
    /// Window length in seconds.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u64).range(1..=3600))]
    window_seconds: u64,
}

pub fn run(args: Args) -> Result<u8> {
    let path = args.output_dir.join("resources.jsonl");
    let text = fs::read_to_string(&path).with_context(|| {
        format!(
            "Read {}; was the check recorded with --resources?",
            path.display()
        )
    })?;
    let report = summarize(&text, args.origin_unix_ms, args.window_seconds)?;
    std::io::stdout().write_all(format!("Samples: {}\n{report}", path.display()).as_bytes())?;
    Ok(0)
}

#[derive(Deserialize)]
struct Reading {
    text: Option<String>,
}

#[derive(Deserialize)]
struct Cgroup {
    path: PathBuf,
    readings: BTreeMap<String, Reading>,
}

#[derive(Deserialize)]
struct Sample {
    unix_ms: u64,
    host: BTreeMap<String, Reading>,
    child_cgroup: Option<Cgroup>,
}

/// Cumulative jiffies over all CPUs.
#[derive(Clone, Copy)]
struct Jiffies {
    busy: u64,
    total: u64,
}

/// The command's cgroup and its cumulative CPU time.
struct CommandCpu {
    cgroup: PathBuf,
    usage_usec: u64,
}

/// One sample's cumulative counters, each absent when its file was unreadable.
struct Point {
    unix_ms: u64,
    host: Option<Jiffies>,
    command: Option<CommandCpu>,
    /// Microseconds in which some task waited for a CPU (`some total=`).
    stall_usec: Option<u64>,
}

impl From<Sample> for Point {
    fn from(sample: Sample) -> Self {
        let text = |reading: Option<&Reading>| reading.and_then(|r| r.text.clone());
        Self {
            unix_ms: sample.unix_ms,
            host: text(sample.host.get(HOST_CPU)).and_then(|t| jiffies(&t)),
            stall_usec: text(sample.host.get(HOST_CPU_PRESSURE)).and_then(|t| stall_usec(&t)),
            command: sample.child_cgroup.and_then(|cgroup| {
                let usage_usec =
                    text(cgroup.readings.get(CGROUP_CPU)).and_then(|t| usage_usec(&t))?;
                Some(CommandCpu {
                    cgroup: cgroup.path,
                    usage_usec,
                })
            }),
        }
    }
}

/// Busy and total jiffies from the aggregate `cpu` line of `/proc/stat`. Guest
/// time is already inside user and nice, so the total stops at steal.
fn jiffies(stat: &str) -> Option<Jiffies> {
    let mut fields = stat.lines().next()?.split_whitespace();
    if fields.next()? != "cpu" {
        return None;
    }
    let values: Vec<u64> = fields
        .take(8)
        .map(|field| field.parse().ok())
        .collect::<Option<_>>()?;
    if values.len() < 8 {
        return None;
    }
    let total: u64 = values.iter().sum();
    let idle = values[3] + values[4];
    Some(Jiffies {
        busy: total - idle,
        total,
    })
}

fn stall_usec(pressure: &str) -> Option<u64> {
    let line = pressure.lines().find(|line| line.starts_with("some "))?;
    line.split_whitespace()
        .find_map(|field| field.strip_prefix("total="))?
        .parse()
        .ok()
}

fn usage_usec(stat: &str) -> Option<u64> {
    stat.lines()
        .find_map(|line| line.strip_prefix("usage_usec "))?
        .trim()
        .parse()
        .ok()
}

/// What changed between two consecutive samples. A figure is absent when
/// either sample lacks it, or, for the command, when they read different cgroups.
struct Interval {
    elapsed_ms: f64,
    host: Option<(f64, f64)>,
    command_usec: Option<f64>,
    stall_usec: Option<f64>,
}

impl Interval {
    fn between(from: &Point, to: &Point) -> Self {
        let host = match (from.host, to.host) {
            (Some(a), Some(b)) if b.total > a.total && b.busy >= a.busy => {
                Some(((b.busy - a.busy) as f64, (b.total - a.total) as f64))
            }
            _ => None,
        };
        let command_usec = match (&from.command, &to.command) {
            (Some(a), Some(b)) if a.cgroup == b.cgroup && b.usage_usec >= a.usage_usec => {
                Some((b.usage_usec - a.usage_usec) as f64)
            }
            _ => None,
        };
        let stall_usec = match (from.stall_usec, to.stall_usec) {
            (Some(a), Some(b)) if b >= a => Some((b - a) as f64),
            _ => None,
        };
        Self {
            elapsed_ms: (to.unix_ms - from.unix_ms) as f64,
            host,
            command_usec,
            stall_usec,
        }
    }
}

/// A window's share of each interval it overlaps, and each interval's own
/// rate for the maxima.
#[derive(Default)]
struct Window {
    sampled_ms: f64,
    host_busy: f64,
    host_total: f64,
    host_max: Option<f64>,
    command_usec: f64,
    command_elapsed_ms: f64,
    command_max: Option<f64>,
    stall_usec: f64,
    stall_elapsed_ms: f64,
    stall_max: Option<f64>,
}

impl Window {
    fn add(&mut self, interval: &Interval, share: f64) {
        let elapsed_ms = interval.elapsed_ms * share;
        self.sampled_ms += elapsed_ms;
        if let Some((busy, total)) = interval.host {
            self.host_busy += busy * share;
            self.host_total += total * share;
            raise(&mut self.host_max, 100.0 * busy / total);
        }
        if let Some(usec) = interval.command_usec {
            self.command_usec += usec * share;
            self.command_elapsed_ms += elapsed_ms;
            raise(&mut self.command_max, cores(usec, interval.elapsed_ms));
        }
        if let Some(usec) = interval.stall_usec {
            self.stall_usec += usec * share;
            self.stall_elapsed_ms += elapsed_ms;
            raise(
                &mut self.stall_max,
                percent_of_time(usec, interval.elapsed_ms),
            );
        }
    }

    fn host(&self) -> Option<f64> {
        (self.host_total > 0.0).then(|| 100.0 * self.host_busy / self.host_total)
    }

    fn command(&self) -> Option<f64> {
        (self.command_elapsed_ms > 0.0).then(|| cores(self.command_usec, self.command_elapsed_ms))
    }

    fn stall(&self) -> Option<f64> {
        (self.stall_elapsed_ms > 0.0)
            .then(|| percent_of_time(self.stall_usec, self.stall_elapsed_ms))
    }
}

fn cores(usec: f64, elapsed_ms: f64) -> f64 {
    usec / (1000.0 * elapsed_ms)
}

fn percent_of_time(usec: f64, elapsed_ms: f64) -> f64 {
    usec / (10.0 * elapsed_ms)
}

fn raise(slot: &mut Option<f64>, value: f64) {
    *slot = Some(slot.map_or(value, |current| current.max(value)));
}

fn cell(value: Option<f64>, decimals: usize) -> String {
    value.map_or_else(|| "–".to_string(), |v| format!("{v:.decimals$}"))
}

/// The windows of a complete `resources.jsonl` as Markdown. Only a last line
/// without its newline may be incomplete: the recorder ends every line it
/// finishes, and a killed recorder can leave the last one partial.
pub(super) fn summarize(text: &str, origin: Option<u64>, window_seconds: u64) -> Result<String> {
    let lines: Vec<&str> = text.lines().collect();
    let unfinished = !text.ends_with('\n');
    let mut points: Vec<Point> = Vec::with_capacity(lines.len());
    let mut incomplete_last = false;
    for (index, line) in lines.iter().enumerate() {
        match serde_json::from_str::<Sample>(line) {
            Ok(sample) => points.push(sample.into()),
            Err(_) if unfinished && index > 0 && index + 1 == lines.len() => incomplete_last = true,
            Err(error) => bail!("Sample line {} is not a sample: {error}", index + 1),
        }
    }
    ensure!(points.len() >= 2, "Fewer than two complete samples");
    ensure!(
        points
            .windows(2)
            .all(|pair| pair[0].unix_ms <= pair[1].unix_ms),
        "Samples are out of time order"
    );
    let origin = i64::try_from(origin.unwrap_or(points[0].unix_ms))?;
    let window_seconds = i64::try_from(window_seconds)?;
    let window_ms = window_seconds * 1000;
    let mut windows: BTreeMap<i64, Window> = BTreeMap::new();
    let mut whole = Window::default();
    for pair in points.windows(2) {
        let interval = Interval::between(&pair[0], &pair[1]);
        let (start, end) = (
            i64::try_from(pair[0].unix_ms)? - origin,
            i64::try_from(pair[1].unix_ms)? - origin,
        );
        if end == start {
            continue;
        }
        whole.add(&interval, 1.0);
        let mut from = start;
        while from < end {
            let index = from.div_euclid(window_ms);
            let to = end.min((index + 1) * window_ms);
            let share = (to - from) as f64 / (end - start) as f64;
            windows.entry(index).or_default().add(&interval, share);
            from = to;
        }
    }
    let cgroups: BTreeSet<String> = points
        .iter()
        .filter_map(|point| point.command.as_ref())
        .map(|command| command.cgroup.display().to_string())
        .collect();

    let mut out = String::new();
    writeln!(
        out,
        "{} samples from {} to {} Unix ms; windows of {window_seconds} s from {origin} Unix ms.",
        points.len(),
        points[0].unix_ms,
        points[points.len() - 1].unix_ms
    )?;
    if incomplete_last {
        writeln!(out, "The last line is incomplete and is left out.")?;
    }
    if cgroups.is_empty() {
        writeln!(out, "No sample read the command's cgroup.")?;
    }
    for cgroup in &cgroups {
        writeln!(out, "Command cgroup: {cgroup}")?;
    }
    writeln!(
        out,
        "Columns: docs/agents/measured-and-baseline-checks.md#resource-measurements\n\n| From s | To s | Sampled s | Host CPU % | Host CPU max % | Command CPU cores | Command CPU max cores | CPU pressure % | CPU pressure max % |\n| --- | --- | --- | --- | --- | --- | --- | --- | --- |"
    )?;
    for (index, window) in &windows {
        writeln!(
            out,
            "| {} | {} | {:.1} | {} | {} | {} | {} | {} | {} |",
            index * window_seconds,
            (index + 1) * window_seconds,
            window.sampled_ms / 1000.0,
            cell(window.host(), 1),
            cell(window.host_max, 1),
            cell(window.command(), 2),
            cell(window.command_max, 2),
            cell(window.stall(), 1),
            cell(window.stall_max, 1)
        )?;
    }
    writeln!(
        out,
        "\nWhole record: host CPU {} % (max {} %), command CPU {} cores (max {}), CPU pressure {} % (max {} %).",
        cell(whole.host(), 1),
        cell(whole.host_max, 1),
        cell(whole.command(), 2),
        cell(whole.command_max, 2),
        cell(whole.stall(), 1),
        cell(whole.stall_max, 1)
    )?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::super::{
        CGROUP_CPU, Cgroup as WrittenCgroup, HOST_CPU, HOST_CPU_PRESSURE,
        Reading as WrittenReading, Sample as Written,
    };
    use super::summarize;
    use std::{collections::BTreeMap, path::PathBuf};

    fn reading(text: &str) -> WrittenReading {
        WrittenReading {
            text: Some(text.to_string()),
            error: None,
        }
    }

    /// A sample line exactly as the recorder writes it.
    fn line(unix_ms: u128, stat: &str, stall_usec: u64, usage_usec: Option<u64>) -> String {
        let host = BTreeMap::from([
            (HOST_CPU, reading(stat)),
            (
                HOST_CPU_PRESSURE,
                reading(&format!(
                    "some avg10=9.99 avg60=0.00 avg300=0.00 total={stall_usec}\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n"
                )),
            ),
            (
                "/proc/meminfo",
                WrittenReading {
                    text: None,
                    error: Some("unread".to_string()),
                },
            ),
        ]);
        let child_cgroup = usage_usec.map(|usage| WrittenCgroup {
            path: PathBuf::from("/sys/fs/cgroup/app.slice/terminal.scope"),
            readings: BTreeMap::from([(
                CGROUP_CPU,
                reading(&format!("usage_usec {usage}\nuser_usec 1\nsystem_usec 1\n")),
            )]),
        });
        let sample = Written {
            unix_ms,
            host,
            child_cgroup_source: None,
            child_cgroup,
        };
        serde_json::to_string(&sample).unwrap()
    }

    fn rows(report: &str) -> Vec<&str> {
        report
            .lines()
            .filter(|line| {
                line.starts_with("| ") && !line.starts_with("| From") && !line.starts_with("| ---")
            })
            .collect()
    }

    #[test]
    fn windows_report_host_command_and_pressure_from_counter_differences() {
        let text = [
            line(1_000, "cpu  100 0 100 800 0 0 0 0 0 0", 0, Some(0)),
            line(
                2_000,
                "cpu  150 0 150 900 0 0 0 0 7 0",
                300_000,
                Some(1_500_000),
            ),
            line(
                3_000,
                "cpu  160 0 160 1080 0 0 0 0 9 0",
                350_000,
                Some(2_000_000),
            ),
        ]
        .join("\n");
        let report = summarize(&text, None, 1).unwrap();
        assert_eq!(
            rows(&report),
            [
                "| 0 | 1 | 1.0 | 50.0 | 50.0 | 1.50 | 1.50 | 30.0 | 30.0 |",
                "| 1 | 2 | 1.0 | 10.0 | 10.0 | 0.50 | 0.50 | 5.0 | 5.0 |",
            ],
            "{report}"
        );
        assert!(
            report.contains(
                "Whole record: host CPU 30.0 % (max 50.0 %), command CPU 1.00 cores (max 1.50), CPU pressure 17.5 % (max 30.0 %)."
            ),
            "{report}"
        );
        assert!(report.contains("Command cgroup: /sys/fs/cgroup/app.slice/terminal.scope"));
    }

    #[test]
    fn intervals_are_shared_between_windows_by_time_and_a_partial_last_line_is_left_out() {
        let text = [
            line(1_000, "cpu  100 0 100 800 0 0 0 0 0 0", 0, None),
            line(2_000, "cpu  150 0 150 900 0 0 0 0 0 0", 0, None),
            line(3_000, "cpu  160 0 160 1080 0 0 0 0 0 0", 0, None),
            "{\"unix_ms\":4000,\"host\":".to_string(),
        ]
        .join("\n");
        let report = summarize(&text, Some(2_500), 1).unwrap();
        assert_eq!(
            rows(&report),
            [
                "| -2 | -1 | 0.5 | 50.0 | 50.0 | – | – | 0.0 | 0.0 |",
                "| -1 | 0 | 1.0 | 30.0 | 50.0 | – | – | 0.0 | 0.0 |",
                "| 0 | 1 | 0.5 | 10.0 | 10.0 | – | – | 0.0 | 0.0 |",
            ],
            "{report}"
        );
        assert!(report.contains("The last line is incomplete and is left out."));
        assert!(report.contains("No sample read the command's cgroup."));
    }

    #[test]
    fn a_damaged_finished_line_and_a_single_sample_are_refused() {
        let first = line(1_000, "cpu  1 0 1 8 0 0 0 0 0 0", 0, None);
        let middle = [first.as_str(), "not json", first.as_str()].join("\n");
        let error = summarize(&middle, None, 10).unwrap_err().to_string();
        assert!(error.contains("line 2"), "{error}");
        let finished_last = format!("{first}\n{first}\nnot json\n");
        let error = summarize(&finished_last, None, 10).unwrap_err().to_string();
        assert!(error.contains("line 3"), "{error}");
        let error = summarize(&first, None, 10).unwrap_err().to_string();
        assert!(error.contains("Fewer than two"), "{error}");
    }
}
