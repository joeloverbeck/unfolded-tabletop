//! Move a consumer's pin of this repository to another revision. `ABOUT`, which
//! `move-pin --help` prints, describes what it changes and refuses.
//!
//! What no test holds: the lockfile refresh, which needs the network to fetch the new
//! revision. The tests hold every manifest edit and every refusal.

use anyhow::{Context, Result, bail};
use clap::Parser;
use std::{
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
};

/// What `--help` prints above the options.
const ABOUT: &str = "\
Move a consumer's pin of unfolded-tabletop to another revision.

It reads every Cargo.toml that Git tracks in the checkout, finds each line naming
https://github.com/joeloverbeck/unfolded-tabletop, and replaces that line's rev with
the new one. Then it refreshes each Cargo.lock that Git tracks there, with
`cargo metadata`, so the consumer's `--locked` checks pass. It prints each file it
changed.

It refuses, and changes nothing, when the checkout is not a Git working tree, when the
revision is not a full 40-character commit hash, when no manifest pins this repository,
when a line names it without a rev on the same line, or when the pins disagree: every
manifest must move together. With --align, pins that disagree, such as a manifest left
behind when the others last moved, all move to the revision. A checkout already at the
revision only has its lockfiles refreshed.

Exit status: 0 when the pins and lockfiles are at the revision, 1 otherwise.";

const REPOSITORY: &str = "https://github.com/joeloverbeck/unfolded-tabletop";

#[derive(Parser)]
#[command(
    name = "move-pin",
    about = "Move a consumer's pin of unfolded-tabletop to another revision.",
    long_about = ABOUT
)]
struct Args {
    /// The consumer's checkout.
    checkout: PathBuf,
    /// The full commit hash to pin.
    revision: String,
    /// Move pins that disagree to the revision as well, instead of refusing.
    #[arg(long)]
    align: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();
    match move_pin(&args.checkout, &args.revision, args.align) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("move-pin: {error:#}");
            ExitCode::from(1)
        }
    }
}

/// One manifest's pins: where each sits and what it pins now.
struct Manifest {
    path: PathBuf,
    text: String,
    /// Byte range of each pinned revision in `text`, with its line number.
    pins: Vec<(std::ops::Range<usize>, usize)>,
}

fn move_pin(checkout: &Path, revision: &str, align: bool) -> Result<()> {
    if revision.len() != 40 || !revision.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{revision:?} is not a full 40-character commit hash");
    }
    let mut manifests = Vec::new();
    for path in tracked(checkout, "Cargo.toml")? {
        let text = std::fs::read_to_string(checkout.join(&path))
            .with_context(|| format!("cannot read {}", path.display()))?;
        let pins = pins(&path, &text)?;
        if !pins.is_empty() {
            manifests.push(Manifest { path, text, pins });
        }
    }
    let mut current: Vec<(String, String)> = Vec::new();
    for manifest in &manifests {
        for (range, line) in &manifest.pins {
            current.push((
                format!("{}:{line}", manifest.path.display()),
                manifest.text[range.clone()].to_owned(),
            ));
        }
    }
    let Some((_, old)) = current.first() else {
        bail!(
            "no Cargo.toml that Git tracks in {} pins {REPOSITORY}",
            checkout.display()
        );
    };
    if !align && current.iter().any(|(_, rev)| rev != old) {
        let listed: Vec<String> = current
            .iter()
            .map(|(place, rev)| format!("  {place}: {rev}"))
            .collect();
        bail!(
            "the pins disagree, so nothing moved:\n{}\n\
             To move every one of them to {revision} anyway, run again with --align.",
            listed.join("\n")
        );
    }
    if current.iter().all(|(_, rev)| rev == revision) {
        println!("already pinned at {revision}");
    } else {
        for manifest in &manifests {
            let mut text = manifest.text.clone();
            let mut was: Vec<&str> = Vec::new();
            for (range, _) in manifest.pins.iter().rev() {
                let rev = &manifest.text[range.clone()];
                if !was.contains(&rev) {
                    was.insert(0, rev);
                }
                text.replace_range(range.clone(), revision);
            }
            std::fs::write(checkout.join(&manifest.path), text)
                .with_context(|| format!("cannot write {}", manifest.path.display()))?;
            println!(
                "{}: {} pin(s) {} -> {revision}",
                manifest.path.display(),
                manifest.pins.len(),
                was.join(", ")
            );
        }
    }
    for lock in tracked(checkout, "Cargo.lock")? {
        let manifest = checkout.join(&lock).with_file_name("Cargo.toml");
        let status = Command::new("cargo")
            .args(["metadata", "--format-version", "1", "--manifest-path"])
            .arg(&manifest)
            .stdout(Stdio::null())
            .status()
            .context("cannot run cargo")?;
        if !status.success() {
            bail!("cargo metadata could not refresh {}", lock.display());
        }
        println!("refreshed {}", lock.display());
    }
    Ok(())
}

/// Every file of this name that Git tracks in the checkout, relative to it.
fn tracked(checkout: &Path, name: &str) -> Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(checkout)
        .args(["ls-files", "-z", "--"])
        .arg(format!(":(glob)**/{name}"))
        .output()
        .context("cannot run git")?;
    if !output.status.success() {
        bail!(
            "{} is not a Git working tree: {}",
            checkout.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(output
        .stdout
        .split(|&byte| byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| PathBuf::from(String::from_utf8_lossy(path).into_owned()))
        .collect())
}

/// The pinned revision on each line that names this repository.
fn pins(path: &Path, text: &str) -> Result<Vec<(std::ops::Range<usize>, usize)>> {
    let mut pins = Vec::new();
    let mut start = 0;
    for (index, line) in text.split_inclusive('\n').enumerate() {
        if line.contains(&format!("\"{REPOSITORY}\"")) {
            let Some(range) = revision_in(line) else {
                bail!(
                    "{}:{} names {REPOSITORY} without a rev on the same line",
                    path.display(),
                    index + 1
                );
            };
            pins.push((start + range.start..start + range.end, index + 1));
        }
        start += line.len();
    }
    Ok(pins)
}

/// The byte range of the quoted value of `rev` in one line, written `rev = "<hash>"`
/// with any spacing around `=`.
fn revision_in(line: &str) -> Option<std::ops::Range<usize>> {
    let mut from = 0;
    while let Some(found) = line[from..].find("rev") {
        let key = from + found;
        from = key + 3;
        let before = line[..key].chars().next_back();
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            continue;
        }
        let rest = line[from..].trim_start();
        let Some(rest) = rest.strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim_start();
        let Some(value) = rest.strip_prefix('"') else {
            continue;
        };
        let begin = line.len() - value.len();
        let end = begin + value.find('"')?;
        return Some(begin..end);
    }
    None
}
