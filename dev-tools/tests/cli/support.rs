use std::{env, path::PathBuf, process::Command};

pub const RUN_CLI: &str = "--run-cli";

/// One `run_cli` invocation in its own process; arguments follow the program name.
pub fn tool() -> Command {
    let mut command = Command::new(fixture());
    command.arg(RUN_CLI);
    command
}

/// This test executable, which also serves as the child a recorded check runs.
pub fn fixture() -> PathBuf {
    env::current_exe().unwrap()
}
