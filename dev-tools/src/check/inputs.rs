//! Declared command input and fresh output files; content validity belongs to the consumer.
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Stdio,
};

const MAX_STDIN_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Default, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub(super) enum Input {
    #[default]
    Null,
    File {
        source: PathBuf,
        snapshot: PathBuf,
        sha256: String,
        bytes: u64,
    },
}

pub(super) fn prepare(source: Option<&Path>, directory: &Path) -> Result<(Input, Stdio)> {
    let Some(source) = source else {
        return Ok((Input::Null, Stdio::null()));
    };
    let source = fs::canonicalize(source).context("Resolve --stdin-file")?;
    ensure!(
        fs::metadata(&source)?.is_file(),
        "--stdin-file must be a regular file"
    );
    let mut bytes = Vec::new();
    File::open(&source)?
        .take(MAX_STDIN_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_STDIN_BYTES,
        "--stdin-file exceeds 16 MiB"
    );
    let snapshot = directory.join("stdin.bin");
    let mut file = File::create_new(&snapshot)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::set_permissions(&snapshot, fs::Permissions::from_mode(0o400))?;
    let input = Input::File {
        source,
        sha256: crate::digest(&snapshot)?,
        bytes: bytes.len() as u64,
        snapshot: snapshot.clone(),
    };
    Ok((input, File::open(snapshot)?.into()))
}

#[derive(Serialize)]
pub(super) struct ExpectedFile {
    path: PathBuf,
    sha256: Option<String>,
}

impl ExpectedFile {
    pub(super) fn new(path: &Path) -> Result<Self> {
        Ok(Self {
            path: std::path::absolute(path)?,
            sha256: None,
        })
    }

    pub(super) fn require_absent(&self) -> Result<()> {
        ensure!(
            fs::symlink_metadata(&self.path)
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound),
            "Expected output must be a fresh path before starting the command: {}",
            self.path.display()
        );
        Ok(())
    }

    pub(super) fn verify(&mut self) -> Result<()> {
        let metadata = fs::symlink_metadata(&self.path).with_context(|| {
            format!(
                "Command did not produce expected file {}",
                self.path.display()
            )
        })?;
        ensure!(
            metadata.is_file() && metadata.len() > 0,
            "Expected output must be a nonempty regular file: {}",
            self.path.display()
        );
        self.sha256 = Some(crate::digest(&self.path)?);
        Ok(())
    }
}
