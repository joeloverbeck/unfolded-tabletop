//! Bounded, identity-checked collection reads extracted from Arkham: relative-path
//! and regular-file checks, bounded and SHA-256-verified reads, bounded PNG crops,
//! and publication of a prepared directory that never replaces an existing output.
//! Consumers own which assets they require and what a missing or changed one means.
//!
//! Every function here refuses with an [`Error`]: its [`Cause`] as data, and the
//! path it concerns. Its `Display` gives the default wording, which names no path.
use sha2::{Digest, Sha256};
use std::{
    fmt, fs, io,
    io::Read,
    path::{Component, Path, PathBuf},
};

/// Why an asset was refused, and the path it concerns: the asset's path under its
/// root, the root, the relative path as given, or the output. Bytes in hand, a
/// crop and a decode concern no path of their own.
#[derive(Debug)]
pub struct Error {
    path: Option<PathBuf>,
    cause: Cause,
}

impl Error {
    fn at(path: &Path, cause: impl Into<Cause>) -> Self {
        Self {
            path: Some(path.to_owned()),
            cause: cause.into(),
        }
    }

    /// The path the refusal concerns, when it concerns one.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Why the asset was refused.
    pub fn cause(&self) -> &Cause {
        &self.cause
    }
}

impl From<Cause> for Error {
    fn from(cause: Cause) -> Self {
        Self { path: None, cause }
    }
}

/// The cause alone, except for [`Cause::Unpublished`], whose wording names its output.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.cause, &self.path) {
            (Cause::Unpublished(error), Some(output)) => write!(
                f,
                "Could not publish {} without replacing existing data: {error}",
                output.display()
            ),
            (cause, _) => cause.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.source()
    }
}

/// The cause of an [`Error`]. `Display` gives the default wording.
#[derive(Debug)]
#[non_exhaustive]
pub enum Cause {
    /// The path is not an ordinary relative path inside the collection.
    NotRelative,
    /// The root is not a directory, or is a symbolic link.
    RootNotDirectory,
    /// The asset, or a directory above it, is not a regular file or directory, or
    /// is a symbolic link.
    NotRegularFile,
    /// The file is longer than its limit.
    TooLarge,
    /// The file's size differs from the selected asset's.
    SizeDiffers,
    /// The bytes' size or SHA-256 differs from the selected asset's.
    Differs,
    /// The crop lies outside its declared atlas, or the atlas exceeds the decoding bound.
    CropOutside,
    /// The atlas could not be decoded. The detail is the decoder's.
    Undecodable(String),
    /// The decoded atlas does not have its declared dimensions.
    WrongDimensions,
    /// The cropped face could not be encoded. The detail is the encoder's.
    Unencodable(String),
    /// The output exists, or the stage could not be moved there.
    Unpublished(io::Error),
    /// The output path has no name to resolve.
    InvalidOutput,
    /// The filesystem failed.
    Io(io::Error),
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotRelative => {
                f.write_str("Asset paths must be ordinary relative paths inside the collection.")
            }
            Self::RootNotDirectory => {
                f.write_str("The root must be a directory, not a symbolic link.")
            }
            Self::NotRegularFile => f.write_str(
                "Assets must be regular files with no symbolic links in their relative paths.",
            ),
            Self::TooLarge => f.write_str("File exceeds its expected size limit."),
            Self::SizeDiffers => f.write_str("File size differs from the selected asset."),
            Self::Differs => f.write_str("File size or SHA-256 differs from the selected asset."),
            Self::CropOutside => {
                f.write_str("The selected PNG crop lies outside its bounded atlas.")
            }
            Self::Undecodable(detail) => {
                write!(f, "The verified PNG atlas could not be decoded: {detail}")
            }
            Self::WrongDimensions => {
                f.write_str("The verified PNG atlas has unexpected dimensions.")
            }
            Self::Unencodable(detail) => {
                write!(f, "The selected PNG face could not be encoded: {detail}")
            }
            Self::Unpublished(error) => {
                write!(
                    f,
                    "Could not publish without replacing existing data: {error}"
                )
            }
            Self::InvalidOutput => f.write_str("Invalid output path."),
            Self::Io(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Cause {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unpublished(error) | Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for Cause {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

/// A game's asset error as one sentence. Superseded by [`Error`], which this
/// package's functions return; it converts to this with the same text.
#[deprecated(
    note = "match `unfolded_assets::Error`'s cause, and keep a sentence error of your own if your game needs one"
)]
#[derive(Debug)]
pub struct AssetError(pub String);

#[allow(deprecated)]
impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

#[allow(deprecated)]
impl std::error::Error for AssetError {}

#[allow(deprecated)]
impl From<Error> for AssetError {
    fn from(error: Error) -> Self {
        Self(error.to_string())
    }
}

#[allow(deprecated)]
impl From<io::Error> for AssetError {
    fn from(error: io::Error) -> Self {
        Self(error.to_string())
    }
}

#[allow(deprecated)]
impl From<serde_json::Error> for AssetError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

/// `value` as a path of ordinary components only, inside the collection.
pub fn relative_path(value: &str) -> Result<&Path, Error> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || !path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(Error::at(path, Cause::NotRelative));
    }
    Ok(path)
}

/// The asset at `relative` under `root`, when it and every directory above it is
/// regular, with no symbolic link. A refusal concerns the asset's whole path, or
/// the root when the root is refused.
pub fn regular_file(root: &Path, relative: &str) -> Result<PathBuf, Error> {
    let at_root = |cause: Cause| Error::at(root, cause);
    if !fs::symlink_metadata(root)
        .map_err(|error| at_root(error.into()))?
        .is_dir()
    {
        return Err(at_root(Cause::RootNotDirectory));
    }
    let relative = relative_path(relative)?;
    let asset = root.join(relative);
    let mut path = root.to_path_buf();
    let mut parts = relative.components().peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        let kind = fs::symlink_metadata(&path)
            .map_err(|error| Error::at(&asset, error))?
            .file_type();
        if kind.is_symlink()
            || if parts.peek().is_some() {
                !kind.is_dir()
            } else {
                !kind.is_file()
            }
        {
            return Err(Error::at(&asset, Cause::NotRegularFile));
        }
    }
    Ok(path)
}

/// The file at `path`, refused when it is longer than `limit`.
pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|file| file.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|error| Error::at(path, error))?;
    if bytes.len() as u64 > limit {
        return Err(Error::at(path, Cause::TooLarge));
    }
    Ok(bytes)
}

/// Whether `bytes` have the expected length and lowercase hex SHA-256. A refusal
/// concerns no path.
pub fn verify_bytes(bytes: &[u8], expected_length: u64, expected_hash: &str) -> Result<(), Error> {
    if bytes.len() as u64 != expected_length
        || format!("{:x}", Sha256::digest(bytes)) != expected_hash
    {
        return Err(Cause::Differs.into());
    }
    Ok(())
}

/// The asset at `path` under `root`, read as [`regular_file`] admits it, refused
/// unless its length and SHA-256 are the selected asset's. Every refusal concerns
/// the asset's whole path, or the root when the root is refused.
pub fn read_verified(root: &Path, path: &str, length: u64, hash: &str) -> Result<Vec<u8>, Error> {
    let path = regular_file(root, path)?;
    if fs::metadata(&path)
        .map_err(|error| Error::at(&path, error))?
        .len()
        != length
    {
        return Err(Error::at(&path, Cause::SizeDiffers));
    }
    let bytes = read_bounded(&path, length)?;
    verify_bytes(&bytes, length, hash).map_err(|error| Error::at(&path, error.cause))?;
    Ok(bytes)
}

/// Explicit bounds for a selected PNG atlas face.
pub struct PngCrop {
    pub source_width: u32,
    pub source_height: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub fn crop_png(bytes: &[u8], crop: PngCrop) -> Result<Vec<u8>, Error> {
    let PngCrop {
        source_width,
        source_height,
        x,
        y,
        width,
        height,
    } = crop;
    // Limit decoding before allocation, and reject changed dimensions or
    // out-of-bounds crops.
    const MAX_DIMENSION: u32 = 1024;
    if source_width == 0
        || source_height == 0
        || source_width > MAX_DIMENSION
        || source_height > MAX_DIMENSION
        || width == 0
        || height == 0
        || x.checked_add(width).is_none_or(|end| end > source_width)
        || y.checked_add(height).is_none_or(|end| end > source_height)
    {
        return Err(Cause::CropOutside.into());
    }
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(source_width);
    limits.max_image_height = Some(source_height);
    limits.max_alloc = Some(16 * 1024 * 1024);
    reader.limits(limits);
    let atlas = reader
        .decode()
        .map_err(|error| Cause::Undecodable(error.to_string()))?;
    if atlas.width() != source_width || atlas.height() != source_height {
        return Err(Cause::WrongDimensions.into());
    }
    let face = atlas.crop_imm(x, y, width, height).into_rgba8();
    let mut output = Vec::new();
    use image::{
        ImageEncoder,
        codecs::png::{CompressionType, FilterType, PngEncoder},
    };
    PngEncoder::new_with_quality(&mut output, CompressionType::Fast, FilterType::NoFilter)
        .write_image(
            face.as_raw(),
            width,
            height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|error| Cause::Unencodable(error.to_string()))?;
    Ok(output)
}

/// Publish an already verified collection without replacing any destination.
pub fn publish_directory(stage: &Path, output: &Path) -> Result<(), Error> {
    // Ubuntu's atomic no-replace rename protects even a destination created by
    // another process after the initial check. An incomplete directory is never published.
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        stage,
        rustix::fs::CWD,
        output,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| Error::at(output, Cause::Unpublished(error.into())))?;
    Ok(())
}

/// `output` as an absolute path with its existing part resolved through links and
/// the rest kept as given. Nothing is created. A refusal concerns `output`.
pub fn resolve_output(output: &Path) -> Result<PathBuf, Error> {
    let refused = |cause: Cause| Error::at(output, cause);
    let mut existing = std::path::absolute(output).map_err(|error| refused(error.into()))?;
    let mut suffix = Vec::new();
    while !existing
        .try_exists()
        .map_err(|error| refused(error.into()))?
    {
        suffix.push(
            existing
                .file_name()
                .ok_or_else(|| refused(Cause::InvalidOutput))?
                .to_os_string(),
        );
        if !existing.pop() {
            return Err(refused(Cause::InvalidOutput));
        }
    }
    let mut resolved = existing
        .canonicalize()
        .map_err(|error| refused(error.into()))?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}
