//! The child a recorded check runs. Behaviour follows Arkham's record-check fixture
//! (`tools/tests/fixtures/child.rs` at c73485c3f066442664073d276fa4d419a3e75cf3).

use std::{
    env,
    ffi::OsString,
    fs,
    io::{Read, Write},
    process::{Command, ExitCode},
    thread,
    time::Duration,
};

pub const RECORD_CHECK: &str = "--fixture-record-check";
pub const DESCENDANT: &str = "--fixture-descendant";

pub fn record_check(mut args: impl Iterator<Item = OsString>) -> ExitCode {
    let mode = args.next().unwrap().into_string().unwrap();
    if let Some(marker) = env::var_os("FIXTURE_STARTED") {
        fs::write(marker, "started").unwrap();
    }
    println!("RECORD_CHECK_STDOUT");
    eprintln!("RECORD_CHECK_STDERR");
    println!("cwd={}", env::current_dir().unwrap().display());
    println!("env={}", env::var("FIXTURE_RECORD_ENV").unwrap_or_default());
    if env::var_os("FIXTURE_DESCENDANT").is_some() {
        // The descendant is left running on purpose: the recorder must clean it up.
        #[expect(clippy::zombie_processes, reason = "the recorder owns its cleanup")]
        let child = Command::new(env::current_exe().unwrap())
            .arg(DESCENDANT)
            .spawn()
            .unwrap();
        fs::write("descendant.pid", child.id().to_string()).unwrap();
    }
    println!("RECORD_CHECK_READY");
    if mode == "stdin" {
        if let Some(path) = env::var_os("FIXTURE_REPLACE_STDIN_SOURCE") {
            fs::write(path, "replaced after launch").unwrap();
        }
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes).unwrap();
        if !bytes.is_empty() {
            fs::write(env::var_os("FIXTURE_STDIN_OUTPUT").unwrap(), bytes).unwrap();
        }
        let code = env::var("FIXTURE_EXIT_CODE").unwrap_or_else(|_| "0".into());
        return ExitCode::from(code.parse::<u8>().unwrap());
    }
    if let Some(log) = env::var_os("FIXTURE_REMOVE_LOG") {
        fs::remove_file(log).unwrap();
    }
    if env::var_os("FIXTURE_LARGE_FAILURE").is_some() {
        println!("error: RETAINED_FAILURE_DIAGNOSTIC");
        println!("{}", "assertion payload 雨".repeat(100_000));
        for line in 0..200 {
            eprintln!("error: diagnostic {line}");
        }
        println!("FULL_LOG_END");
    }
    if let Some(path) = env::var_os("FIXTURE_CHANGE_SOURCE") {
        fs::write(path, "changed during check").unwrap();
    }
    std::io::stdout().flush().unwrap();
    std::io::stderr().flush().unwrap();
    if mode == "wait" {
        descendant();
    }
    ExitCode::from(mode.parse::<u8>().unwrap())
}

pub fn descendant() -> ! {
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
