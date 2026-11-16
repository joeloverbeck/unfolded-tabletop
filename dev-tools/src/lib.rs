//! Development support extracted from Arkham d0b57009ba037ca8756a4f2756dcb550d8634773.
//! Consumers own check commands, publication policy and game-specific preparation.
//! This crate is independent of the tabletop runtime and rendering dependencies,
//! and the runtime crate does not depend on it.
//!
//! A consumer may embed [`reader`], [`check`], [`process`] and [`digest`], or call
//! [`run_cli`] for the same reading and recording commands in a process of their
//! own, which installs its interrupt handler there. Reader arguments keep their
//! explicit byte limits, paging cursors and source hashes; recording keeps complete
//! logs, source fingerprints, interruption and child cleanup. What they produce is
//! mechanical evidence, not authority to accept or publish a result. The
//! `compare-captures` binary is not part of [`run_cli`], so the commands a consumer
//! wraps through it are unchanged.
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::{
    ffi::OsString,
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub mod check;
pub mod process;
pub mod reader;

#[derive(Parser)]
#[command(name = "unfolded-dev-tools")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ReadContext(reader::Args),
    ReadContextBatch(reader::BatchArgs),
    SearchContext(reader::SearchArgs),
    CheckSummary(check::summary::Args),
    ResourceSummary(check::resources::summary::Args),
    RecordCheck(check::CheckArgs),
}

/// Run the shared commands in a dedicated process. Installs one interrupt handler;
/// callers embedding individual modules instead own their handler and exit policy.
/// `args` includes `argv[0]`, following clap's parsing convention.
pub fn run_cli(args: impl IntoIterator<Item = OsString>) -> u8 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let code = error.exit_code() as u8;
            let _ = error.print();
            return code;
        }
    };
    let failure = if matches!(cli.command, Command::RecordCheck(_)) {
        1
    } else {
        2
    };
    let result = process::install_interrupt_handler().and_then(|()| match cli.command {
        Command::ReadContext(args) => write_page(reader::page(args)),
        Command::ReadContextBatch(args) => write_page(reader::batch(args)),
        Command::SearchContext(args) => write_page(reader::search(args)),
        Command::CheckSummary(args) => check::summary::run(args),
        Command::ResourceSummary(args) => check::resources::summary::run(args),
        Command::RecordCheck(args) => check::run(args),
    });
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("unfolded-dev-tools: {error:#}");
            if process::interrupted() { 130 } else { failure }
        }
    }
}

fn write_page(page: std::result::Result<String, String>) -> Result<u8> {
    std::io::stdout().write_all(page.map_err(anyhow::Error::msg)?.as_bytes())?;
    Ok(0)
}

pub fn digest(path: &Path) -> Result<String> {
    let mut file = File::open(path).with_context(|| format!("Cannot read {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
