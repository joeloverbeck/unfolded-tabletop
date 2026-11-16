//! The save record held at its interface, with a test-owned game format: the
//! envelope, its checksum, the write baseline, the unchanged shortcut, recovery
//! validation, recovery into a fresh destination, exclusive creation, archives
//! and the size limit. Each game's own adapter and wording stay in that game.

use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use unfolded_record_replacement::{
    archives, lock_path, previous_path,
    save::{self, Baseline, Error, Format, Identity, Write},
};

/// A test game: a note whose text must not be empty, saved under one revision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Note {
    text: String,
}

struct Notes {
    revision: &'static str,
}

impl Format for Notes {
    type Payload = Note;
    const LIMIT: u64 = 256;
    fn identity(&self, _: &Note) -> Identity {
        [("kind", "note"), ("revision", self.revision)]
            .into_iter()
            .collect()
    }
    fn accepts(&self, identity: &Identity) -> bool {
        identity.get("kind") == Some("note") && identity.get("revision") == Some(self.revision)
    }
    fn validate(&self, note: &Note) -> Result<(), String> {
        if note.text.is_empty() {
            return Err("A note needs text.".into());
        }
        Ok(())
    }
}

const NOTES: Notes = Notes { revision: "7" };

fn note(text: &str) -> Note {
    Note { text: text.into() }
}

/// Write `text` to a fresh record, returning its baseline.
fn saved(path: &Path, text: &str) -> Baseline {
    let mut baseline = Baseline::default();
    save::write(&NOTES, path, &mut baseline, &note(text), Write::Save).unwrap();
    baseline
}

#[test]
fn a_saved_record_is_the_documented_envelope_and_loads_back() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let baseline = saved(&path, "hello");

    // The checksum oracle is `sha256sum` of the JSON array
    // ["unfolded-save-1",{"kind":"note","revision":"7"},{"text":"hello"}].
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        concat!(
            r#"{"record":"unfolded-save-1","identity":{"kind":"note","revision":"7"},"#,
            r#""payload":{"text":"hello"},"#,
            r#""checksum":"935c196420f3c32b16660ef5a38204771f1e3f828d001f05b6e146e25c5d99ab"}"#
        )
    );
    let (loaded, loaded_baseline) = save::load(&NOTES, &path).unwrap().unwrap();
    assert_eq!(loaded, note("hello"));
    assert_eq!(loaded_baseline, baseline);
}

/// Assert `result` refused as `matches`, and that `path` still holds `bytes`.
fn assert_refused<T: std::fmt::Debug>(
    result: Result<T, Error>,
    matches: fn(&Error) -> bool,
    path: &Path,
    bytes: &[u8],
) {
    let error = result.unwrap_err();
    assert!(matches(&error), "{error:?}");
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn an_envelope_with_an_unknown_field_is_refused_and_left_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "hello");
    let mut record: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    record["note"] = "extra".into();
    let bytes = serde_json::to_vec(&record).unwrap();
    fs::write(&path, &bytes).unwrap();
    assert_refused(
        save::load(&NOTES, &path),
        |e| matches!(e, Error::Malformed(detail) if detail.contains("unknown field")),
        &path,
        &bytes,
    );
}

#[test]
fn a_different_identity_is_unsupported_and_names_what_was_recorded() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "hello");
    let bytes = fs::read(&path).unwrap();
    let later = Notes { revision: "8" };
    let error = save::load(&later, &path).unwrap_err();
    let Error::Unsupported(Some(identity)) = &error else {
        panic!("{error:?}");
    };
    assert_eq!(identity.get("revision"), Some("7"));
    assert_eq!(identity.to_string(), "kind note, revision 7");
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn a_record_in_a_previous_format_is_unsupported_and_left_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    // The shape Arkham wrote before this envelope.
    let bytes = br#"{"format_version":3,"kind":"note","source_revision":"7","app_version":"0.1.0","payload":{"text":"hello"},"checksum":"0"}"#;
    fs::write(&path, bytes).unwrap();
    assert_refused(
        save::load(&NOTES, &path),
        |e| matches!(e, Error::Unsupported(None)),
        &path,
        bytes,
    );
    // Nor will a save replace it, and the refusal still says why.
    assert_refused(
        save::write(
            &NOTES,
            &path,
            &mut Baseline::default(),
            &note("new"),
            Write::Save,
        ),
        |e| matches!(e, Error::Unsupported(None)),
        &path,
        bytes,
    );
}

#[test]
fn a_checksum_mismatch_is_refused_and_left_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "hello");
    let bytes = fs::read_to_string(&path)
        .unwrap()
        .replace("hello", "jello")
        .into_bytes();
    fs::write(&path, &bytes).unwrap();
    assert_refused(
        save::load(&NOTES, &path),
        |e| matches!(e, Error::Checksum),
        &path,
        &bytes,
    );
}

#[test]
fn a_sealed_payload_that_fails_the_game_check_is_unreproducible() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "hello");
    let (identity, mut payload) = save::unseal(&fs::read(&path).unwrap()).unwrap();
    payload["text"] = "".into();
    let bytes = save::seal(&identity, &payload);
    fs::write(&path, &bytes).unwrap();
    assert_refused(
        save::load(&NOTES, &path),
        |e| matches!(e, Error::Unreproducible(detail) if detail == "A note needs text."),
        &path,
        &bytes,
    );
}

#[test]
fn an_invalid_payload_is_refused_before_anything_is_written() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let error = save::write(
        &NOTES,
        &path,
        &mut Baseline::default(),
        &note(""),
        Write::Save,
    )
    .unwrap_err();
    assert!(matches!(&error, Error::Invalid(detail) if detail == "A note needs text."));
    assert!(!dir.path().join("notes.json").exists());
}

#[test]
fn a_stale_writer_is_refused_and_the_later_save_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "first");
    let (_, mut early) = save::load(&NOTES, &path).unwrap().unwrap();
    let (_, mut later) = save::load(&NOTES, &path).unwrap().unwrap();
    save::write(&NOTES, &path, &mut later, &note("later"), Write::Save).unwrap();
    let kept = fs::read(&path).unwrap();
    let before = early.clone();
    assert_refused(
        save::write(&NOTES, &path, &mut early, &note("early"), Write::Save),
        |e| matches!(e, Error::Stale),
        &path,
        &kept,
    );
    assert_eq!(early, before, "a refusal keeps the baseline");
    // Once it has seen the later record, the writer may continue.
    let (_, mut reopened) = save::load(&NOTES, &path).unwrap().unwrap();
    save::write(&NOTES, &path, &mut reopened, &note("early"), Write::Save).unwrap();
}

#[test]
fn a_replaced_record_that_disappeared_is_missing_but_a_save_creates_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    fs::remove_file(&path).unwrap();
    let before = baseline.clone();
    let error =
        save::write(&NOTES, &path, &mut baseline, &note("next"), Write::Replace).unwrap_err();
    assert!(matches!(error, Error::Missing), "{error:?}");
    assert!(!path.exists());
    assert_eq!(baseline, before);

    // Saving a loaded session to a new path creates it there.
    let elsewhere = dir.path().join("elsewhere.json");
    save::write(
        &NOTES,
        &elsewhere,
        &mut baseline,
        &note("next"),
        Write::Save,
    )
    .unwrap();
    assert_eq!(
        save::load(&NOTES, &elsewhere).unwrap().unwrap(),
        (note("next"), baseline)
    );
}

#[test]
fn an_unchanged_write_writes_nothing_and_validates_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    save::write(&NOTES, &path, &mut baseline, &note("second"), Write::Save).unwrap();
    let current = fs::read(&path).unwrap();
    fs::write(previous_path(&path), b"damaged recovery").unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();

    let before = baseline.clone();
    for mode in [Write::Save, Write::Replace] {
        save::write(&NOTES, &path, &mut baseline, &note("second"), mode).unwrap();
    }
    assert_eq!(fs::read(&path).unwrap(), current);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"damaged recovery");
    assert_eq!(baseline, before);

    // A writer that never saw the record is stale even with the same payload.
    assert_refused(
        save::write(
            &NOTES,
            &path,
            &mut Baseline::default(),
            &note("second"),
            Write::Save,
        ),
        |e| matches!(e, Error::Stale),
        &path,
        &current,
    );
}

#[test]
fn the_same_record_formatted_differently_is_unchanged_and_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    let mut reformatted = fs::read(&path).unwrap();
    reformatted.extend_from_slice(b"\n \t\n");
    fs::write(&path, &reformatted).unwrap();
    fs::write(previous_path(&path), b"damaged recovery").unwrap();

    save::write(&NOTES, &path, &mut baseline, &note("first"), Write::Replace).unwrap();
    assert_eq!(fs::read(&path).unwrap(), reformatted);
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"damaged recovery");
    // The baseline is now the reformatted bytes, so the writer may continue.
    fs::remove_file(previous_path(&path)).unwrap();
    save::write(
        &NOTES,
        &path,
        &mut baseline,
        &note("second"),
        Write::Replace,
    )
    .unwrap();
    assert_eq!(fs::read(previous_path(&path)).unwrap(), reformatted);
}

#[test]
fn the_previous_record_is_validated_before_rotation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    let first = fs::read(&path).unwrap();
    save::write(&NOTES, &path, &mut baseline, &note("second"), Write::Save).unwrap();
    assert_eq!(fs::read(previous_path(&path)).unwrap(), first);
    let second = fs::read(&path).unwrap();

    fs::write(previous_path(&path), b"damaged recovery").unwrap();
    assert_refused(
        save::write(&NOTES, &path, &mut baseline, &note("third"), Write::Save),
        |e| matches!(e, Error::Malformed(_)),
        &path,
        &second,
    );
    assert_eq!(fs::read(previous_path(&path)).unwrap(), b"damaged recovery");
}

#[test]
fn recovery_lands_the_previous_record_in_a_fresh_destination() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("notes.json");
    let mut baseline = saved(&source, "first");
    save::write(&NOTES, &source, &mut baseline, &note("second"), Write::Save).unwrap();
    let active = fs::read(&source).unwrap();
    let previous = fs::read(previous_path(&source)).unwrap();

    let fresh = dir.path().join("recovered.json");
    let (payload, mut recovered) =
        save::recover(&NOTES, &source, &fresh, |_| Ok::<_, Error>(())).unwrap();
    assert_eq!(payload, note("first"));
    assert_eq!(fs::read(&fresh).unwrap(), previous);
    assert_eq!(fs::read(&source).unwrap(), active);
    assert_eq!(fs::read(previous_path(&source)).unwrap(), previous);
    assert!(!previous_path(&fresh).exists());
    // The recovered record continues under its own baseline.
    save::write(&NOTES, &fresh, &mut recovered, &note("third"), Write::Save).unwrap();

    for same in [
        source.clone(),
        previous_path(&source),
        dir.path().join(".").join("notes.json"),
    ] {
        let error = save::recover(&NOTES, &source, &same, |_| Ok::<_, Error>(())).unwrap_err();
        assert!(matches!(error, Error::SameRecord), "{same:?}: {error:?}");
    }
    assert_refused(
        save::recover(&NOTES, &source, &fresh, |_| Ok::<_, Error>(())),
        |e| matches!(e, Error::Occupied),
        &fresh,
        &fs::read(&fresh).unwrap(),
    );
    assert_eq!(fs::read(&source).unwrap(), active);
}

#[test]
fn an_exclusive_create_is_refused_where_a_record_or_its_previous_is() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "first");
    let bytes = fs::read(&path).unwrap();
    assert_refused(
        save::write(
            &NOTES,
            &path,
            &mut Baseline::default(),
            &note("new"),
            Write::Create,
        ),
        |e| matches!(e, Error::Occupied),
        &path,
        &bytes,
    );

    let orphan = dir.path().join("orphan.json");
    fs::write(previous_path(&orphan), &bytes).unwrap();
    for mode in [Write::Create, Write::Save] {
        let error = save::write(
            &NOTES,
            &orphan,
            &mut Baseline::default(),
            &note("new"),
            mode,
        )
        .unwrap_err();
        assert!(matches!(error, Error::Orphaned), "{mode:?}: {error:?}");
        assert!(!orphan.exists());
    }
}

#[test]
fn an_archive_returns_its_path_and_keeps_the_record_it_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    let first = fs::read(&path).unwrap();

    let archive = save::write(&NOTES, &path, &mut baseline, &note("fresh"), Write::Archive)
        .unwrap()
        .unwrap();
    assert_eq!(archives(&path).unwrap(), vec![archive.clone()]);
    assert_eq!(fs::read(&archive).unwrap(), first);
    assert_eq!(fs::read(previous_path(&path)).unwrap(), first);
    assert_eq!(save::load(&NOTES, &path).unwrap().unwrap().0, note("fresh"));

    let fresh = fs::read(&path).unwrap();
    assert_refused(
        save::write(
            &NOTES,
            &path,
            &mut Baseline::default(),
            &note("other"),
            Write::Archive,
        ),
        |e| matches!(e, Error::Stale),
        &path,
        &fresh,
    );
    let gone = dir.path().join("gone.json");
    let error =
        save::write(&NOTES, &gone, &mut baseline, &note("other"), Write::Archive).unwrap_err();
    assert!(matches!(error, Error::Missing), "{error:?}");
    assert_eq!(archives(&path).unwrap(), vec![archive]);
}

#[test]
fn an_oversized_record_is_refused_both_ways() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let long = note(&"x".repeat(256));
    let error =
        save::write(&NOTES, &path, &mut Baseline::default(), &long, Write::Save).unwrap_err();
    assert!(matches!(error, Error::TooLarge), "{error:?}");
    assert!(!path.exists());

    let bytes = vec![b' '; 257];
    fs::write(&path, &bytes).unwrap();
    assert_refused(
        save::load(&NOTES, &path),
        |e| matches!(e, Error::TooLarge),
        &path,
        &bytes,
    );
}

#[test]
fn a_held_lock_makes_the_save_busy() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    let bytes = fs::read(&path).unwrap();
    let lock = fs::File::open(lock_path(&path)).unwrap();
    lock.try_lock().unwrap();
    assert_refused(
        save::write(&NOTES, &path, &mut baseline, &note("next"), Write::Save),
        |e| matches!(e, Error::Busy(_)),
        &path,
        &bytes,
    );
}

#[test]
fn a_link_is_not_loaded_or_recovered() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = saved(&path, "first");
    save::write(&NOTES, &path, &mut baseline, &note("second"), Write::Save).unwrap();
    let link = dir.path().join("link.json");
    std::os::unix::fs::symlink(&path, &link).unwrap();
    let error = save::load(&NOTES, &link).unwrap_err();
    assert!(matches!(error, Error::NotRegularFile), "{error:?}");

    let elsewhere = dir.path().join("elsewhere.json");
    std::os::unix::fs::symlink(previous_path(&path), previous_path(&elsewhere)).unwrap();
    let error = save::recover(&NOTES, &elsewhere, &dir.path().join("fresh.json"), |_| {
        Ok::<_, Error>(())
    })
    .unwrap_err();
    assert!(matches!(error, Error::NotRegularFile), "{error:?}");
}

#[test]
fn an_absent_record_loads_as_none() {
    let dir = tempfile::tempdir().unwrap();
    assert!(
        save::load(&NOTES, &dir.path().join("none.json"))
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_recovery_the_game_refuses_creates_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("notes.json");
    let mut baseline = saved(&source, "first");
    save::write(&NOTES, &source, &mut baseline, &note("second"), Write::Save).unwrap();
    let fresh = dir.path().join("recovered.json");
    let refusal = save::recover(&NOTES, &source, &fresh, |recovered: &Note| {
        assert_eq!(recovered, &note("first"));
        Err::<(), Box<dyn std::error::Error>>("Not for this launch.".into())
    });
    assert_eq!(refusal.unwrap_err().to_string(), "Not for this launch.");
    assert!(!fresh.exists());
    assert!(!unfolded_record_replacement::lock_path(&fresh).exists());
}

#[test]
fn a_baseline_holds_only_the_payload_last_loaded_or_saved() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let baseline = saved(&path, "first");
    assert!(baseline.holds(&NOTES, &note("first")).unwrap());
    assert!(!baseline.holds(&NOTES, &note("second")).unwrap());
    assert!(!Baseline::default().holds(&NOTES, &note("first")).unwrap());
    let error = baseline.holds(&NOTES, &note("")).unwrap_err();
    assert!(matches!(error, Error::Invalid(_)), "{error:?}");
}

/// A format that accepts every revision of a note but writes revision 7.
struct AnyRevision;
impl Format for AnyRevision {
    type Payload = Note;
    const LIMIT: u64 = 256;
    fn identity(&self, note: &Note) -> Identity {
        NOTES.identity(note)
    }
    fn accepts(&self, identity: &Identity) -> bool {
        identity.get("kind") == Some("note")
    }
    fn validate(&self, note: &Note) -> Result<(), String> {
        NOTES.validate(note)
    }
}

#[test]
fn envelope_and_payload_faults_are_named_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    saved(&path, "hello");
    let (identity, payload) = save::unseal(&fs::read(&path).unwrap()).unwrap();
    let record: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();

    let mut missing = record.clone();
    missing.as_object_mut().unwrap().remove("checksum");
    let mut undecodable = payload.clone();
    undecodable["text"] = 7.into();
    let later: Identity = [("kind", "note"), ("revision", "8")].into_iter().collect();
    // An unaccepted identity is unsupported even when its checksum is wrong too.
    let mut unaccepted = record.clone();
    unaccepted["identity"]["revision"] = "9".into();
    unaccepted["checksum"] = "wrong".into();

    type Load = fn(&Path) -> Result<Option<(Note, Baseline)>, Error>;
    let strict: Load = |path| save::load(&NOTES, path);
    let loose: Load = |path| save::load(&AnyRevision, path);
    type Refused = fn(&Error) -> bool;
    let cases: [(Vec<u8>, Load, Refused); 4] = [
        (
            serde_json::to_vec(&missing).unwrap(),
            strict,
            |e| matches!(e, Error::Malformed(detail) if detail.contains("missing field")),
        ),
        (save::seal(&identity, &undecodable), strict, |e| {
            matches!(e, Error::Unreproducible(_))
        }),
        (
            save::seal(&later, &payload),
            loose,
            |e| matches!(e, Error::Unreproducible(detail) if detail.contains("identity")),
        ),
        (serde_json::to_vec(&unaccepted).unwrap(), strict, |e| {
            matches!(e, Error::Unsupported(Some(_)))
        }),
    ];
    for (bytes, load, refused) in cases {
        fs::write(&path, &bytes).unwrap();
        let error = load(&path).unwrap_err();
        assert!(refused(&error), "{error:?}");
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn a_replace_with_an_unset_baseline_creates_and_an_archive_always_writes() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notes.json");
    let mut baseline = Baseline::default();
    save::write(&NOTES, &path, &mut baseline, &note("first"), Write::Replace).unwrap();
    let first = fs::read(&path).unwrap();
    let archive = save::write(&NOTES, &path, &mut baseline, &note("first"), Write::Archive)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(archive).unwrap(), first);
    assert_eq!(fs::read(previous_path(&path)).unwrap(), first);
}

#[test]
fn recovery_names_a_missing_previous_an_orphaned_destination_and_a_linked_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("notes.json");
    let ok = |_: &Note| Ok::<_, Error>(());

    let error = save::recover(&NOTES, &source, &dir.path().join("fresh.json"), ok).unwrap_err();
    assert!(
        matches!(&error, Error::Io(cause) if cause.kind() == std::io::ErrorKind::NotFound),
        "{error:?}"
    );

    let mut baseline = saved(&source, "first");
    save::write(&NOTES, &source, &mut baseline, &note("second"), Write::Save).unwrap();
    let orphan = dir.path().join("orphan.json");
    fs::write(previous_path(&orphan), b"someone's history").unwrap();
    let error = save::recover(&NOTES, &source, &orphan, ok).unwrap_err();
    assert!(matches!(error, Error::Orphaned), "{error:?}");
    assert!(!orphan.exists());

    let link = dir.path().join("link");
    std::os::unix::fs::symlink(dir.path(), &link).unwrap();
    let error = save::recover(&NOTES, &source, &link.join("notes.json"), ok).unwrap_err();
    assert!(matches!(error, Error::SameRecord), "{error:?}");
}
