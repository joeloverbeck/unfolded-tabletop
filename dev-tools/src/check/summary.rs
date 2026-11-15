//! Diagnostic excerpts only: the child status in run.json remains authoritative.
use anyhow::{Context, Result, ensure};
use clap::Args as ClapArgs;
use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

#[derive(ClapArgs)]
pub struct Args {
    /// A retained check.log (also accepts ordinary compiler/test logs).
    log: PathBuf,
    /// Maximum UTF-8 output bytes, including metadata and omission markers.
    #[arg(long, default_value_t = 6000, value_parser = clap::value_parser!(u32).range(512..=12000))]
    max_bytes: u32,
}

pub fn run(args: Args) -> Result<u8> {
    let path = std::fs::canonicalize(&args.log).context("Locate full check log")?;
    let heading = format!(
        "Full log: {}\nDiagnostic excerpts only; consult run.json for status.\n",
        path.display()
    );
    let text = summarize(
        BufReader::new(File::open(path)?),
        &heading,
        args.max_bytes as usize,
    )?;
    std::io::stdout().write_all(text.as_bytes())?;
    Ok(0)
}

pub(super) fn summarize(mut reader: impl BufRead, heading: &str, budget: usize) -> Result<String> {
    const FOOTER: &str = "Excerpts may omit diagnostics; full output remains in the log above.\n";
    ensure!(
        heading.len() + FOOTER.len() + 100 <= budget,
        "Output budget too small for log path"
    );
    let body_budget = budget - heading.len() - FOOTER.len();
    let mut details = Excerpts::new(body_budget);
    let mut failures = Excerpts::new(body_budget);
    let mut tail = VecDeque::new();
    let mut following = 0;
    let mut failure_following = 0;
    let mut number = 0;
    while let Some((line, clipped)) = read_line(&mut reader)? {
        number += 1;
        let line = clean(&line);
        let diagnostic = line.contains("panicked at")
            || line.starts_with("error")
            || line.starts_with("Diff in ");
        let entry = format!(
            "{number}: {line}{}\n",
            if clipped { " … [line truncated]" } else { "" }
        );
        if tail.len() == 6 {
            tail.pop_front();
        }
        tail.push_back(entry.clone());
        if diagnostic {
            following = 6;
        }
        if following > 0 {
            details.push(&entry);
            following -= 1;
        }
        if line.contains("... FAILED") || line.starts_with("test result: FAILED") {
            failure_following = 6;
        }
        if failure_following > 0 {
            failures.push(&entry);
            failure_following -= 1;
        }
    }
    // Progress rows often precede panic details by hundreds of lines. Reserve
    // the budget for those details; use failure rows only when details are absent.
    let mut excerpts = if !details.text.is_empty() {
        details
    } else {
        failures
    };
    if excerpts.text.is_empty() {
        excerpts.push("No recognized diagnostic markers; bounded tail follows.\n");
        for entry in tail {
            excerpts.push(&entry);
        }
    }
    if excerpts.omitted {
        excerpts.text.push_str("[output budget reached]\n");
    }
    Ok(format!("{heading}{}{FOOTER}", excerpts.text))
}

struct Excerpts {
    text: String,
    budget: usize,
    omitted: bool,
}

impl Excerpts {
    fn new(budget: usize) -> Self {
        Self {
            text: String::new(),
            budget,
            omitted: false,
        }
    }

    fn push(&mut self, entry: &str) {
        if !self.omitted
            && self.text.len() + entry.len() + "[output budget reached]\n".len() <= self.budget
        {
            self.text.push_str(entry);
        } else {
            self.omitted = true;
        }
    }
}

// Consume arbitrarily long lines in bounded memory. Lossy decoding preserves a
// readable diagnostic even for a split UTF-8 sequence or binary compiler output.
fn read_line(reader: &mut impl BufRead) -> Result<Option<(Vec<u8>, bool)>> {
    let mut line = Vec::new();
    let mut total = 0;
    loop {
        let bytes = reader.fill_buf()?;
        if bytes.is_empty() {
            return Ok((total != 0).then_some((line, total > 240)));
        }
        let end = bytes.iter().position(|b| *b == b'\n');
        let length = end.unwrap_or(bytes.len());
        line.extend_from_slice(&bytes[..length.min(240 - line.len())]);
        total += length;
        reader.consume(length + usize::from(end.is_some()));
        if end.is_some() {
            return Ok(Some((line, total > 240)));
        }
    }
}

fn clean(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let mut chars = text.chars().peekable();
    let mut output = String::new();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        } else if !ch.is_control() || ch == '\t' {
            output.push(ch);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn passing_suites_and_failure_progress_cannot_displace_later_panic_details() {
        let mut log = "test result: ok. 100 passed\n".repeat(100);
        log.push_str(&"test early_progress ... FAILED\n".repeat(100));
        log.push_str("thread 'actual_failure' panicked at src/actual.rs:42:9:\nassertion failed: expected preserved state\n");
        let result = summarize(log.as_bytes(), "Full log: /tmp/check.log\n", 512).unwrap();
        assert!(result.len() <= 512);
        for expected in [
            "actual_failure",
            "src/actual.rs:42:9",
            "expected preserved state",
        ] {
            assert!(result.contains(expected), "{result}");
        }
        assert!(!result.contains("early_progress"));
        assert!(!result.contains("100 passed"));
    }
    #[test]
    fn giant_assertions_keep_failure_identity_location_and_budget() {
        let log = format!(
            "running tests\nthread 'sealed_reveal' panicked at src/trays.rs:42:9:\nassertion failed\nleft: {}\nright: unchanged\ntest result: FAILED\n",
            "☠".repeat(100_000)
        );
        for budget in [512, 1000, 6000] {
            let result = summarize(log.as_bytes(), "Full log: /tmp/check.log\n", budget).unwrap();
            assert!(result.len() <= budget);
            assert!(result.contains("sealed_reveal"));
            assert!(result.contains("src/trays.rs:42:9"));
        }
        assert!(
            summarize(log.as_bytes(), "log\n", 6000)
                .unwrap()
                .contains("[line truncated]")
        );
    }
    #[test]
    fn compiler_formatting_and_unknown_failures_remain_inspectable() {
        for marker in [
            "\x1b[31merror[E0425]\x1b[0m: missing symbol",
            "Diff in src/test.rs:42:",
            "test actual_path ... FAILED",
        ] {
            let result = summarize(
                format!("{marker}\n  --> src/test.rs:42\n").as_bytes(),
                "log\n",
                6000,
            )
            .unwrap();
            assert!(result.contains("src/test.rs:42"));
            assert!(!result.contains("\x1b"));
            assert!(!result.contains("No recognized"));
        }
        let result = summarize(
            b"ordinary output\nexternal tool refused\n".as_slice(),
            "log\n",
            512,
        )
        .unwrap();
        assert!(result.contains("No recognized"));
        assert!(result.contains("external tool refused"));
    }
}
