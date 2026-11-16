//! A game's save record on the locked replacement lifecycle.
//!
//! # What this module owns
//!
//! One envelope shape, the same for every game:
//!
//! ```json
//! {"record":"unfolded-save-1","identity":{"kind":"…"},"payload":{…},"checksum":"…"}
//! ```
//!
//! `checksum` is the lowercase hex SHA-256 of the compact JSON array
//! `["unfolded-save-1", identity, payload]`. A record is written as compact JSON.
//! This module also owns the size bound's application, the [`Baseline`] of the
//! bytes last loaded or saved, the shortcut that writes nothing when nothing
//! changed, validation of the `.previous` record before rotation, recovery from it
//! into a fresh destination, and the regular-file check before every read.
//!
//! # What the game supplies
//!
//! A [`Format`]: its payload type, the [`Identity`] values that mark a compatible
//! save and which identities it accepts, its validity check and its size limit.
//! The game decides when a write is exclusive ([`Write::Create`]) and when the
//! current record is archived ([`Write::Archive`]), and words every [`Error`].
//!
//! # Loading
//!
//! [`load`] refuses, leaving the file unchanged, in this order:
//! a link, directory or other non-regular file ([`Error::NotRegularFile`]); a
//! record over the limit ([`Error::TooLarge`]); bytes that are not JSON
//! ([`Error::Malformed`]); JSON that is not this envelope, such as a game's
//! previous format ([`Error::Unsupported`]); an envelope with an unknown or
//! missing field ([`Error::Malformed`]); an identity the game does not accept
//! ([`Error::Unsupported`]); a checksum mismatch ([`Error::Checksum`]); and a
//! payload that does not decode, fails the game's check, or carries a different
//! identity from the one the game gives it ([`Error::Unreproducible`]).
//!
//! # Writing
//!
//! [`write()`] validates the payload, seals it, and installs it under the lock. With
//! the current record's bytes in hand:
//!
//! - [`Write::Save`] and [`Write::Replace`] replace a record that matches the
//!   baseline. A record that does not, or exists while the baseline is unset, is
//!   [`Error::Stale`], even when the payload is the same: a writer that has not
//!   seen the record is refused. A record that is not a save this build loads is
//!   refused as [`load`] would refuse it instead.
//! - When the payload is the record already there, they write nothing and
//!   validate no `.previous`. The baseline is the record's JSON, so the same
//!   record formatted differently is unchanged too.
//! - Where no record is, [`Write::Save`] creates one as [`Write::Create`] does,
//!   whatever the baseline: a session loaded from one path can be saved to a new
//!   one. [`Write::Replace`] refuses as [`Error::Missing`] when the baseline is
//!   set: the record it continues is gone. With the baseline unset it creates.
//! - [`Write::Create`] creates only where no record is: an existing one is
//!   [`Error::Occupied`], and a `.previous` record without one is
//!   [`Error::Orphaned`].
//! - [`Write::Archive`] keeps the current record in a restart archive and
//!   returns its path. It needs the record ([`Error::Missing`]) to match the
//!   baseline ([`Error::Stale`], or the refusal [`load`] would give a record this
//!   build cannot load), and writes even when the payload is the same. A game
//!   that also needs the record to be a particular payload asks
//!   [`Baseline::holds`] first.
//!
//! Before a record is rotated into `.previous`, an existing `.previous` must load
//! under the game's format. On success the baseline becomes the record now on
//! disk; on any refusal it is unchanged.
//!
//! # What no test holds
//!
//! Durability through power loss, parent-directory syncing and multi-file
//! atomicity, as for [`replace`](crate::replace). Two processes holding the same
//! baseline are kept apart by the lock and the baseline check, but a test holds
//! only the stale refusal, not every interleaving. A baseline matches the same
//! JSON value, so records that differ only in whitespace match; whether a record
//! with its keys reordered matches depends on serde_json's `preserve_order`
//! feature being off, and two records whose numbers read as the same value match.
//! Either case can only cause an extra stale refusal or an unchanged write, never
//! a write over a record its holder has not seen as that value.

use crate::{Fault, Replacement, previous_path, read_bounded, replace_bounded};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// The envelope tag every record carries; a record without it is unsupported.
const ENVELOPE_TAG: &str = "unfolded-save-1";

/// The names and values that identify a compatible save, such as a game's kind,
/// rules revision or content digest. Written as a JSON object with sorted keys.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Identity(BTreeMap<String, String>);

impl Identity {
    /// The value recorded under `name`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.0.get(name).map(String::as_str)
    }
}

impl<K: Into<String>, V: Into<String>> FromIterator<(K, V)> for Identity {
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        Self(
            iter.into_iter()
                .map(|(name, value)| (name.into(), value.into()))
                .collect(),
        )
    }
}

/// Each `name value` pair, separated by commas, in name order.
impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, (name, value)) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{name} {value}")?;
        }
        Ok(())
    }
}

/// A game's saved format.
pub trait Format {
    /// What the game saves.
    type Payload: Serialize + DeserializeOwned;
    /// The largest record, in bytes, this game reads or writes.
    const LIMIT: u64;
    /// The identity written with `payload`.
    fn identity(&self, payload: &Self::Payload) -> Identity;
    /// Whether a record with `identity` is a save this build can load.
    fn accepts(&self, identity: &Identity) -> bool;
    /// The game's validity check, run before a payload is written and after one is
    /// loaded. The error is the game's own detail.
    fn validate(&self, payload: &Self::Payload) -> Result<(), String>;
}

/// The record last loaded or saved, kept as the SHA-256 digest of its JSON
/// written out again, so the same record formatted differently matches. Bytes
/// that are not JSON are digested as they are. A default baseline is unset: its
/// holder has not seen any record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Baseline(Option<[u8; 32]>);

impl Baseline {
    fn of(bytes: &[u8]) -> Self {
        let digest = match serde_json::from_slice::<serde_json::Value>(bytes) {
            Ok(value) => {
                Sha256::digest(serde_json::to_vec(&value).expect("a JSON value always serialises"))
            }
            Err(_) => Sha256::digest(bytes),
        };
        Self(Some(digest.into()))
    }
    fn matches(&self, bytes: &[u8]) -> bool {
        *self == Self::of(bytes)
    }
    fn is_set(&self) -> bool {
        self.0.is_some()
    }
    /// Whether this baseline is the record `format` would write for `payload`:
    /// whether its holder's payload is what it last loaded or saved. Nothing is
    /// read from disk; [`write()`] then checks the baseline against the record.
    pub fn holds<F: Format>(&self, format: &F, payload: &F::Payload) -> Result<bool, Error> {
        Ok(*self == Self::of(&encode(format, payload)?))
    }
}

/// How [`write()`] installs a payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Write {
    /// Replace the record the baseline saw, or create one where no record is.
    Save,
    /// Replace only the record the baseline saw: one that is gone is missing.
    Replace,
    /// Create a record only where none is.
    Create,
    /// Replace the record the baseline saw, keeping it in a restart archive.
    Archive,
}

/// Why a save record was refused. Every refusal leaves the record, its `.previous`
/// and the baseline unchanged, except where [`write()`] notes partial effects of a
/// failed installation. The game words each one; `Display` gives plain defaults.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// The record is not this envelope, or its identity is not accepted. The
    /// identity is the one recorded, when the envelope has one.
    Unsupported(Option<Identity>),
    /// The bytes are not JSON, or the envelope has an unknown or missing field.
    Malformed(String),
    /// The checksum does not match the record's other fields.
    Checksum,
    /// The payload does not decode, fails the game's check, or does not match its
    /// identity. The detail is the decoder's or the game's.
    Unreproducible(String),
    /// The payload to be written fails the game's check. Nothing was written.
    Invalid(String),
    /// The record is not the one the baseline saw.
    Stale,
    /// The record the baseline saw, or that an archive needs, is gone.
    Missing,
    /// An exclusive creation found a record already there.
    Occupied,
    /// A `.previous` record exists without its active record.
    Orphaned,
    /// A recovery destination is the source record or its `.previous`.
    SameRecord,
    /// Another writer holds the lock. The detail is the operating system's.
    Busy(String),
    /// A record, or the payload to be written, exceeds the format's limit.
    TooLarge,
    /// The record is a link, directory or other non-regular file.
    NotRegularFile,
    /// The filesystem failed.
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported(Some(identity)) => write!(f, "Unsupported save ({identity})."),
            Self::Unsupported(None) => f.write_str("Unsupported save."),
            Self::Malformed(detail) => write!(f, "Invalid save: {detail}"),
            Self::Checksum => f.write_str("Save checksum mismatch."),
            Self::Unreproducible(detail) | Self::Invalid(detail) => f.write_str(detail),
            Self::Stale => f.write_str("A different or later decision is already saved."),
            Self::Missing => f.write_str("The save is missing."),
            Self::Occupied => f.write_str("The save path is already occupied."),
            Self::Orphaned => f.write_str("A previous record exists without an active save."),
            Self::SameRecord => {
                f.write_str("Recovery needs a destination distinct from both source records.")
            }
            Self::Busy(cause) => f.write_str(&Fault::Busy(cause.clone()).words()),
            Self::TooLarge => f.write_str(&Fault::TooLarge.words()),
            Self::NotRegularFile => f.write_str("The save is not a regular file."),
            Self::Io(cause) => cause.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(cause: io::Error) -> Self {
        Self::Io(cause)
    }
}

fn from_fault(fault: Fault) -> Error {
    match fault {
        Fault::TooLarge => Error::TooLarge,
        Fault::Busy(cause) => Error::Busy(cause),
        Fault::NotRegularFile => Error::NotRegularFile,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    record: String,
    identity: Identity,
    payload: serde_json::Value,
    checksum: String,
}

fn checksum(identity: &Identity, payload: &serde_json::Value) -> String {
    let sealed = serde_json::to_vec(&(ENVELOPE_TAG, identity, payload))
        .expect("an identity and a JSON value always serialise");
    format!("{:x}", Sha256::digest(sealed))
}

/// Seal `payload` under `identity` without validating either: the bytes a game's
/// own tests need to show that its validity check refuses a record whose
/// checksum is intact.
pub fn seal(identity: &Identity, payload: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&Envelope {
        record: ENVELOPE_TAG.into(),
        identity: identity.clone(),
        checksum: checksum(identity, payload),
        payload: payload.clone(),
    })
    .expect("an envelope always serialises")
}

/// Open a record's envelope without the game's format: its identity and its
/// payload as JSON, after the checksum. A game's tests use it with [`seal`] to
/// edit a record.
pub fn unseal(bytes: &[u8]) -> Result<(Identity, serde_json::Value), Error> {
    open_envelope(bytes, |_| true)
}

fn open_envelope(
    bytes: &[u8],
    accepts: impl FnOnce(&Identity) -> bool,
) -> Result<(Identity, serde_json::Value), Error> {
    let value: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|cause| Error::Malformed(cause.to_string()))?;
    if value.get("record").and_then(serde_json::Value::as_str) != Some(ENVELOPE_TAG) {
        let recorded = value
            .get("identity")
            .and_then(|identity| serde_json::from_value(identity.clone()).ok());
        return Err(Error::Unsupported(recorded));
    }
    let envelope: Envelope =
        serde_json::from_value(value).map_err(|cause| Error::Malformed(cause.to_string()))?;
    // Identity before checksum: an unaccepted record is unsupported, not damaged.
    if !accepts(&envelope.identity) {
        return Err(Error::Unsupported(Some(envelope.identity)));
    }
    if envelope.checksum != checksum(&envelope.identity, &envelope.payload) {
        return Err(Error::Checksum);
    }
    Ok((envelope.identity, envelope.payload))
}

fn encode<F: Format>(format: &F, payload: &F::Payload) -> Result<Vec<u8>, Error> {
    format.validate(payload).map_err(Error::Invalid)?;
    let value = serde_json::to_value(payload).map_err(|cause| Error::Invalid(cause.to_string()))?;
    Ok(seal(&format.identity(payload), &value))
}

fn decode<F: Format>(format: &F, bytes: &[u8]) -> Result<F::Payload, Error> {
    let (identity, payload) = open_envelope(bytes, |identity| format.accepts(identity))?;
    let payload: F::Payload = serde_json::from_value(payload)
        .map_err(|cause| Error::Unreproducible(cause.to_string()))?;
    format.validate(&payload).map_err(Error::Unreproducible)?;
    if format.identity(&payload) != identity {
        return Err(Error::Unreproducible(
            "The saved identity does not match its payload.".into(),
        ));
    }
    Ok(payload)
}

fn read_record(path: &Path, limit: u64) -> Result<Vec<u8>, Error> {
    if !fs::symlink_metadata(path)?.is_file() {
        return Err(Error::NotRegularFile);
    }
    read_bounded(path, limit, &from_fault)
}

/// Load the record at `path` and the baseline of its bytes, or `None` when no
/// file is there. See the [module](self) for each refusal.
pub fn load<F: Format>(format: &F, path: &Path) -> Result<Option<(F::Payload, Baseline)>, Error> {
    match fs::symlink_metadata(path) {
        Err(cause) if cause.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(cause) => return Err(cause.into()),
        Ok(_) => {}
    }
    let bytes = read_record(path, F::LIMIT)?;
    let payload = decode(format, &bytes)?;
    Ok(Some((payload, Baseline::of(&bytes))))
}

fn previous_exists(path: &Path) -> Result<bool, Error> {
    match fs::symlink_metadata(previous_path(path)) {
        Ok(_) => Ok(true),
        Err(cause) if cause.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(cause) => Err(cause.into()),
    }
}

/// Write `payload` to the record at `path` as `mode` asks, returning the restart
/// archive for [`Write::Archive`]. See the [module](self) for the rules and
/// refusals. As with [`replace`](crate::replace), a failure after the lock was
/// taken can leave an archive or a rotated `.previous` behind.
pub fn write<F: Format>(
    format: &F,
    path: &Path,
    baseline: &mut Baseline,
    payload: &F::Payload,
    mode: Write,
) -> Result<Option<PathBuf>, Error> {
    let bytes = encode(format, payload)?;
    let written = Baseline::of(&bytes);
    let archived = replace_bounded(
        path,
        &bytes,
        F::LIMIT,
        &from_fault,
        |existing| {
            let Some(existing) = existing else {
                return absent(path, baseline, mode);
            };
            if mode == Write::Create {
                return Err(Error::Occupied);
            }
            if !baseline.matches(existing) {
                // Name what is there when it is not a save this build loads.
                decode(format, existing)?;
                return Err(Error::Stale);
            }
            if mode != Write::Archive && *baseline == written {
                return Ok(Replacement::Unchanged);
            }
            Ok(if mode == Write::Archive {
                Replacement::Archive
            } else {
                Replacement::Replace
            })
        },
        |previous| decode(format, &read_record(previous, F::LIMIT)?).map(|_| ()),
    )?;
    *baseline = written;
    Ok(archived)
}

fn absent(path: &Path, baseline: &Baseline, mode: Write) -> Result<Replacement, Error> {
    if mode == Write::Archive || (mode == Write::Replace && baseline.is_set()) {
        return Err(Error::Missing);
    }
    if previous_exists(path)? {
        return Err(Error::Orphaned);
    }
    Ok(Replacement::Create)
}

/// The path with its parent resolved through links, so two spellings of one
/// file compare equal. A missing parent leaves the absolute path as given.
fn location(path: &Path) -> Result<PathBuf, Error> {
    let absolute = std::path::absolute(path)?;
    let (Some(parent), Some(name)) = (absolute.parent(), absolute.file_name()) else {
        return Ok(absolute);
    };
    match parent.canonicalize() {
        Ok(parent) => Ok(parent.join(name)),
        Err(cause) if cause.kind() == io::ErrorKind::NotFound => Ok(absolute),
        Err(cause) => Err(cause.into()),
    }
}

/// Recover the `.previous` record of `source` into a fresh `destination`,
/// returning its payload and the new record's baseline. `admit` is the game's
/// own check of the recovered payload, such as whether it suits this launch; it
/// runs before anything is created, and its refusal is returned as it is.
///
/// `source` and its `.previous` are left unchanged. A destination that is either
/// of them is [`Error::SameRecord`]; one already in use is [`Error::Occupied`] or
/// [`Error::Orphaned`], as for [`Write::Create`]. A missing `.previous` is an
/// [`Error::Io`] whose kind is `NotFound`.
pub fn recover<F: Format, E: From<Error>>(
    format: &F,
    source: &Path,
    destination: &Path,
    admit: impl FnOnce(&F::Payload) -> Result<(), E>,
) -> Result<(F::Payload, Baseline), E> {
    let previous = previous_path(source);
    let fresh = location(destination)?;
    if fresh == location(source)? || fresh == location(&previous)? {
        return Err(Error::SameRecord.into());
    }
    let payload = decode(format, &read_record(&previous, F::LIMIT)?)?;
    admit(&payload)?;
    let mut baseline = Baseline::default();
    write(format, destination, &mut baseline, &payload, Write::Create)?;
    Ok((payload, baseline))
}
