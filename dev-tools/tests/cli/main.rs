//! The shared commands exercised through `run_cli`, each invocation in a dedicated
//! process as consumers run them. This executable is its own harness: started with
//! `--run-cli` it is the command line, and with a fixture argument it is the child a
//! recorded check runs. No consumer binary, compiler or sibling checkout is needed.

use std::{env, ffi::OsString, process::ExitCode, sync::Mutex, thread};

/// One named case per function, so a reported name cannot drift from its function.
macro_rules! cases {
    ($module:literal: $($case:ident),* $(,)?) => {
        &[$((concat!($module, "::", stringify!($case)), $case as fn())),*]
    };
}

mod fixture;
mod reader;
mod record;
mod search;
mod support;

pub type Case = (&'static str, fn());

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    match args.next() {
        Some(mode) if mode == support::RUN_CLI => {
            let argv = std::iter::once(OsString::from("unfolded-dev-tools")).chain(args);
            return ExitCode::from(unfolded_dev_tools::run_cli(argv));
        }
        Some(mode) if mode == fixture::RECORD_CHECK => return fixture::record_check(args),
        Some(mode) if mode == fixture::DESCENDANT => fixture::descendant(),
        _ => {}
    }
    run([reader::CASES, search::CASES, record::CASES].concat())
}

/// Accept the common test-harness arguments: name filters, `--skip`, `--exact`,
/// `--list`, `--ignored` (no case here is ignored) and `--test-threads` or
/// `RUST_TEST_THREADS`; other options are ignored, with their values where they take one.
fn run(cases: Vec<Case>) -> ExitCode {
    let (mut filters, mut skips, mut exact, mut list) = (Vec::new(), Vec::new(), false, false);
    let mut only_ignored = false;
    let mut threads = env::var("RUST_TEST_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--skip" => skips.extend(args.next()),
            "--exact" => exact = true,
            "--list" => list = true,
            "--ignored" => only_ignored = true,
            "--test-threads" => threads = args.next().and_then(|value| value.parse().ok()),
            option if option.starts_with("--test-threads=") => {
                threads = option["--test-threads=".len()..].parse().ok();
            }
            "--format" | "--color" | "--logfile" | "-Z" => {
                args.next();
            }
            option if option.starts_with('-') => {}
            filter => filters.push(filter.to_owned()),
        }
    }
    let matches = |name: &str, pattern: &String| {
        if exact {
            name == pattern
        } else {
            name.contains(pattern.as_str())
        }
    };
    let selected: Vec<Case> = cases
        .into_iter()
        .filter(|(name, _)| filters.is_empty() || filters.iter().any(|f| matches(name, f)))
        .filter(|(name, _)| !skips.iter().any(|s| matches(name, s)))
        .filter(|_| !only_ignored)
        .collect();
    if list {
        for (name, _) in &selected {
            println!("{name}: test");
        }
        return ExitCode::SUCCESS;
    }
    println!("\nrunning {} tests", selected.len());
    let total = selected.len();
    let queue = Mutex::new(selected.into_iter());
    let failures = Mutex::new(Vec::new());
    let workers = threads.filter(|&count| count > 0).unwrap_or_else(|| {
        thread::available_parallelism()
            .map_or(4, |n| n.get())
            .min(8)
    });
    thread::scope(|scope| {
        for _ in 0..workers {
            scope.spawn(|| {
                loop {
                    let next = queue.lock().unwrap().next();
                    let Some((name, case)) = next else { break };
                    // A named thread makes the default panic message name the case.
                    let passed = thread::Builder::new()
                        .name(name.to_owned())
                        .spawn(case)
                        .unwrap()
                        .join()
                        .is_ok();
                    println!("test {name} ... {}", if passed { "ok" } else { "FAILED" });
                    if !passed {
                        failures.lock().unwrap().push(name);
                    }
                }
            });
        }
    });
    let failures = failures.into_inner().unwrap();
    if failures.is_empty() {
        println!("\ntest result: ok. {total} passed; 0 failed\n");
        ExitCode::SUCCESS
    } else {
        println!("\nfailures:");
        for name in &failures {
            println!("    {name}");
        }
        println!(
            "\ntest result: FAILED. {} passed; {} failed\n",
            total - failures.len(),
            failures.len()
        );
        ExitCode::FAILURE
    }
}
