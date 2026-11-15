//! Endpoint fingerprints, not a filesystem lock or proof against edit-and-revert.
use anyhow::{Context, Result, bail};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    ffi::OsStr,
    fs, io,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Default, Serialize)]
pub(super) struct Report {
    root: Option<PathBuf>,
    before_sha256: Option<String>,
    after_sha256: Option<String>,
}

impl Report {
    pub fn start(cwd: &Path, excluded: &Path) -> Result<Self> {
        let output = Command::new("git")
            .current_dir(cwd)
            .args(["rev-parse", "--show-toplevel"])
            .output()?;
        if !output.status.success() {
            // Only an ordinary directory outside Git may omit source evidence.
            if String::from_utf8_lossy(&output.stderr).contains("not a git repository") {
                return Ok(Self::default());
            }
            bail!(
                "Cannot locate check source: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let root = PathBuf::from(OsStr::from_bytes(
            output.stdout.strip_suffix(b"\n").unwrap_or(&output.stdout),
        ));
        let before_sha256 = Some(fingerprint(&root, excluded)?);
        Ok(Self {
            root: Some(root),
            before_sha256,
            after_sha256: None,
        })
    }

    pub fn finish(&mut self, excluded: &Path) -> Result<()> {
        if let Some(root) = &self.root {
            self.after_sha256 = Some(fingerprint(root, excluded)?);
        }
        Ok(())
    }

    pub fn changed(&self) -> bool {
        self.before_sha256 != self.after_sha256
    }
}

fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git").current_dir(root).args(args).output()?;
    if !output.status.success() {
        bail!(
            "Source inventory failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(output.stdout)
}

fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn fingerprint(root: &Path, excluded: &Path) -> Result<String> {
    let inventory = git(
        root,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
    )?;
    let paths: BTreeSet<_> = inventory
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .collect();
    let mut hash = Sha256::new();
    field(&mut hash, b"arkham-check-source-v1");
    for path in paths {
        let full = root.join(OsStr::from_bytes(path));
        if full.starts_with(excluded) {
            continue;
        }
        field(&mut hash, path);
        let meta = match fs::symlink_metadata(&full) {
            Ok(meta) => meta,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                field(&mut hash, b"missing");
                continue;
            }
            Err(error) => return Err(error).with_context(|| format!("Inspect {}", full.display())),
        };
        field(
            &mut hash,
            &(meta.permissions().mode() & 0o777).to_le_bytes(),
        );
        if meta.is_symlink() {
            field(&mut hash, b"symlink");
            field(&mut hash, fs::read_link(&full)?.as_os_str().as_bytes());
        } else if meta.is_file() {
            field(&mut hash, b"file");
            let mut file = fs::File::open(&full)?;
            let mut content = Sha256::new();
            io::copy(&mut file, &mut content)?;
            field(&mut hash, &content.finalize());
        } else {
            bail!(
                "Unsupported source entry {} (including submodules)",
                full.display()
            );
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_permissions_and_symlink_targets_are_source_changes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        git(root, &["init", "-q"]).unwrap();
        let source = root.join("source.rs");
        fs::write(&source, "original").unwrap();
        git(root, &["add", "source.rs"]).unwrap();
        let excluded = root.join("record");
        let original = fingerprint(root, &excluded).unwrap();
        fs::set_permissions(&source, fs::Permissions::from_mode(0o755)).unwrap();
        let executable = fingerprint(root, &excluded).unwrap();
        assert_ne!(original, executable);
        fs::remove_file(&source).unwrap();
        let deleted = fingerprint(root, &excluded).unwrap();
        assert_ne!(executable, deleted);
        std::os::unix::fs::symlink("first-target", &source).unwrap();
        let first = fingerprint(root, &excluded).unwrap();
        assert_ne!(deleted, first);
        fs::remove_file(&source).unwrap();
        std::os::unix::fs::symlink("second-target", &source).unwrap();
        assert_ne!(first, fingerprint(root, &excluded).unwrap());
    }
}
