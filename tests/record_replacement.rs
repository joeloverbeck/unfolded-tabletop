//! File-replacement behavior retained from Arkham at 89be726, exercised through
//! the shared public boundary. Game admission and real driver tests stay there.

use std::fs;
use unfolded_tabletop::record_replacement::{Replacement, previous_path, replace};

#[test]
fn exclusive_creation_preserves_a_file_that_appears_after_admission() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("attempt.json");
    let result = replace(
        &path,
        b"new attempt",
        1024,
        std::io::Error::other,
        |previous| {
            assert!(previous.is_none());
            fs::write(&path, b"competing attempt")?;
            Ok(Replacement::Create)
        },
        |_| Ok(()),
    );
    assert!(result.is_err());
    assert_eq!(fs::read(&path).unwrap(), b"competing attempt");
    assert!(!previous_path(&path).exists());
}

#[test]
fn oversized_candidate_is_refused_before_creating_directories_or_replacing_bytes() {
    let dir = tempfile::tempdir().unwrap();
    for path in [
        dir.path().join("missing/record"),
        dir.path().join("existing"),
    ] {
        if path.file_name().unwrap() == "existing" {
            fs::write(&path, b"keep").unwrap();
        }
        let result = replace(
            &path,
            b"too large",
            4,
            std::io::Error::other,
            |_| panic!("Oversized candidate reached admission"),
            |_| Ok(()),
        );
        assert_eq!(
            result.unwrap_err().to_string(),
            "This file exceeds the bounded experiment's record limit."
        );
    }
    assert!(!dir.path().join("missing").exists());
    assert_eq!(fs::read(dir.path().join("existing")).unwrap(), b"keep");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}
