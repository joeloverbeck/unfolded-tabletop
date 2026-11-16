//! The `move-pin` binary on checkouts each test builds with `git`. None tracks a
//! `Cargo.lock`, so no test refreshes one; every expected manifest and line of output
//! is written out by hand.

use std::{path::Path, process::Command};

const OLD: &str = "9ab6f1ebc3c4162a09681d5d215f26c04cd93cd8";
const NEW: &str = "29de68fb8e18df19facdf41b61727453e04ce5fb";

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

fn run(checkout: &Path, revision: &str) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_move-pin"))
        .arg(checkout)
        .arg(revision)
        .output()
        .unwrap();
    Run {
        code: output.status.code(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

/// A Git working tree holding `tracked` files in its index and `untracked` beside them.
fn checkout(tracked: &[(&str, &str)], untracked: &[(&str, &str)]) -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .arg("-C")
            .arg(directory.path())
            .args(args)
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?}");
    };
    git(&["init", "-q"]);
    for (path, text) in tracked.iter().chain(untracked) {
        let file = directory.path().join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    for (path, _) in tracked {
        git(&["add", path]);
    }
    directory
}

fn read(directory: &tempfile::TempDir, path: &str) -> String {
    std::fs::read_to_string(directory.path().join(path)).unwrap()
}

fn root(rev: &str) -> String {
    format!(
        "[dependencies]\n\
         unfolded-random = {{ git = \"https://github.com/joeloverbeck/unfolded-tabletop\", rev = \"{rev}\" }}\n\
         other = {{ git = \"https://example.com/other\", rev = \"0123\" }}\n\
         \n\
         [patch.crates-io]\n\
         wgpu-hal = {{ git = \"https://github.com/joeloverbeck/unfolded-tabletop\", rev = \"{rev}\" }}\n"
    )
}

fn tools(rev: &str) -> String {
    format!(
        "[dependencies]\n\
         unfolded-dev-tools = {{ git = \"https://github.com/joeloverbeck/unfolded-tabletop\", rev=\"{rev}\", optional = true }}\n"
    )
}

#[test]
fn every_tracked_pin_moves_and_nothing_else_does() {
    let stale = root(OLD);
    let directory = checkout(
        &[
            ("Cargo.toml", &root(OLD)),
            ("tools/Cargo.toml", &tools(OLD)),
        ],
        &[("target/stale/Cargo.toml", &stale)],
    );

    let run = run(directory.path(), NEW);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        format!("Cargo.toml: 2 pin(s) {OLD} -> {NEW}\ntools/Cargo.toml: 1 pin(s) {OLD} -> {NEW}\n")
    );
    assert_eq!(read(&directory, "Cargo.toml"), root(NEW));
    assert_eq!(read(&directory, "tools/Cargo.toml"), tools(NEW));
    assert_eq!(read(&directory, "target/stale/Cargo.toml"), stale);
}

#[test]
fn a_checkout_at_the_revision_is_left_as_it_is() {
    let directory = checkout(&[("Cargo.toml", &root(NEW))], &[]);

    let run = run(directory.path(), NEW);

    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(run.stdout, format!("already pinned at {NEW}\n"));
    assert_eq!(read(&directory, "Cargo.toml"), root(NEW));
}

#[test]
fn pins_that_disagree_are_named_and_nothing_moves() {
    let directory = checkout(
        &[
            ("Cargo.toml", &root(OLD)),
            ("tools/Cargo.toml", &tools(NEW)),
        ],
        &[],
    );

    let run = run(directory.path(), NEW);

    assert_eq!(run.code, Some(1));
    assert_eq!(
        run.stderr,
        format!(
            "move-pin: the pins disagree, so nothing moved:\n  \
             Cargo.toml:2: {OLD}\n  Cargo.toml:6: {OLD}\n  tools/Cargo.toml:2: {NEW}\n"
        )
    );
    assert_eq!(read(&directory, "Cargo.toml"), root(OLD));
    assert_eq!(read(&directory, "tools/Cargo.toml"), tools(NEW));
}

#[test]
fn every_refusal_changes_nothing() {
    let split = "[dependencies]\n\
                 unfolded-random = { git = \"https://github.com/joeloverbeck/unfolded-tabletop\",\n  \
                 rev = \"9ab6f1ebc3c4162a09681d5d215f26c04cd93cd8\" }\n";
    let unpinned =
        "[dependencies]\nother = { git = \"https://example.com/other\", rev = \"0123\" }\n";
    for (manifest, revision, error) in [
        (
            split,
            NEW,
            "move-pin: Cargo.toml:2 names https://github.com/joeloverbeck/unfolded-tabletop without a rev on the same line\n",
        ),
        (
            unpinned,
            NEW,
            "move-pin: no Cargo.toml that Git tracks in {} pins https://github.com/joeloverbeck/unfolded-tabletop\n",
        ),
        (
            &root(OLD),
            "29de68f",
            "move-pin: \"29de68f\" is not a full 40-character commit hash\n",
        ),
    ] {
        let directory = checkout(&[("Cargo.toml", manifest)], &[]);
        let run = run(directory.path(), revision);
        assert_eq!(run.code, Some(1), "{error}");
        assert_eq!(
            run.stderr,
            error.replace("{}", &directory.path().display().to_string())
        );
        assert_eq!(run.stdout, "");
        assert_eq!(read(&directory, "Cargo.toml"), manifest);
    }
}

#[test]
fn a_directory_outside_git_is_refused() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("Cargo.toml"), root(OLD)).unwrap();

    let run = run(directory.path(), NEW);

    assert_eq!(run.code, Some(1));
    assert!(
        run.stderr.starts_with(&format!(
            "move-pin: {} is not a Git working tree",
            directory.path().display()
        )),
        "{}",
        run.stderr
    );
    assert_eq!(read_plain(directory.path()), root(OLD));
}

fn read_plain(directory: &Path) -> String {
    std::fs::read_to_string(directory.join("Cargo.toml")).unwrap()
}
