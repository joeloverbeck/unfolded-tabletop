//! Bounded, identity-checked collection reads extracted from Arkham.
use sha2::{Digest, Sha256};
use std::{
    fmt, fs,
    io::Read,
    path::{Component, Path, PathBuf},
};
#[derive(Debug)]
pub struct AssetError(pub String);

impl fmt::Display for AssetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for AssetError {}

impl From<std::io::Error> for AssetError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<serde_json::Error> for AssetError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

pub fn relative_path(value: &str) -> Result<&Path, AssetError> {
    let path = Path::new(value);
    if value.is_empty()
        || value.contains('\\')
        || !path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(AssetError(
            "Asset paths must be ordinary relative paths inside the collection.".into(),
        ));
    }
    Ok(path)
}

pub fn regular_file(root: &Path, relative: &str) -> Result<PathBuf, AssetError> {
    if !fs::symlink_metadata(root)?.is_dir() {
        return Err(AssetError(
            "The root must be a directory, not a symbolic link.".into(),
        ));
    }
    let relative = relative_path(relative)?;
    let mut path = root.to_path_buf();
    let mut parts = relative.components().peekable();
    while let Some(part) = parts.next() {
        path.push(part);
        let kind = fs::symlink_metadata(&path)?.file_type();
        if kind.is_symlink()
            || if parts.peek().is_some() {
                !kind.is_dir()
            } else {
                !kind.is_file()
            }
        {
            return Err(AssetError(
                "Assets must be regular files with no symbolic links in their relative paths."
                    .into(),
            ));
        }
    }
    Ok(path)
}

pub fn read_bounded(path: &Path, limit: u64) -> Result<Vec<u8>, AssetError> {
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err(AssetError("File exceeds its expected size limit.".into()));
    }
    Ok(bytes)
}

pub fn verify_bytes(
    bytes: &[u8],
    expected_length: u64,
    expected_hash: &str,
) -> Result<(), AssetError> {
    if bytes.len() as u64 != expected_length
        || format!("{:x}", Sha256::digest(bytes)) != expected_hash
    {
        return Err(AssetError(
            "File size or SHA-256 differs from the selected asset.".into(),
        ));
    }
    Ok(())
}

pub fn read_verified(
    root: &Path,
    path: &str,
    length: u64,
    hash: &str,
) -> Result<Vec<u8>, AssetError> {
    let path = regular_file(root, path)?;
    if fs::metadata(&path)?.len() != length {
        return Err(AssetError(
            "File size differs from the selected asset.".into(),
        ));
    }
    let bytes = read_bounded(&path, length)?;
    verify_bytes(&bytes, length, hash)?;
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

pub fn crop_png(bytes: &[u8], crop: PngCrop) -> Result<Vec<u8>, AssetError> {
    let PngCrop {
        source_width,
        source_height,
        x,
        y,
        width,
        height,
    } = crop;
    // The selected 768px clue atlas needs one exact face. Limit decoding
    // before allocation and reject changed dimensions or out-of-bounds crops.
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
        return Err(AssetError(
            "The selected PNG crop lies outside its bounded atlas.".into(),
        ));
    }
    let mut reader =
        image::ImageReader::with_format(std::io::Cursor::new(bytes), image::ImageFormat::Png);
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(source_width);
    limits.max_image_height = Some(source_height);
    limits.max_alloc = Some(16 * 1024 * 1024);
    reader.limits(limits);
    let atlas = reader.decode().map_err(|error| {
        AssetError(format!(
            "The verified PNG atlas could not be decoded: {error}"
        ))
    })?;
    if atlas.width() != source_width || atlas.height() != source_height {
        return Err(AssetError(
            "The verified PNG atlas has unexpected dimensions.".into(),
        ));
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
        .map_err(|error| {
            AssetError(format!(
                "The selected PNG face could not be encoded: {error}"
            ))
        })?;
    Ok(output)
}

/// Publish an already verified collection without replacing any destination.
pub fn publish_directory(stage: &Path, output: &Path) -> Result<(), AssetError> {
    // Ubuntu's atomic no-replace rename protects even a destination created by
    // another process after the initial check. An incomplete directory is never published.
    rustix::fs::renameat_with(
        rustix::fs::CWD,
        stage,
        rustix::fs::CWD,
        output,
        rustix::fs::RenameFlags::NOREPLACE,
    )
    .map_err(|error| {
        AssetError(format!(
            "Could not publish {} without replacing existing data: {error}",
            output.display()
        ))
    })?;
    Ok(())
}
pub fn resolve_output(output: &Path) -> Result<PathBuf, AssetError> {
    let mut existing = std::path::absolute(output)?;
    let mut suffix = Vec::new();
    while !existing.try_exists()? {
        suffix.push(
            existing
                .file_name()
                .ok_or_else(|| AssetError("Invalid output path.".into()))?
                .to_os_string(),
        );
        if !existing.pop() {
            return Err(AssetError("Invalid output path.".into()));
        }
    }
    let mut resolved = existing.canonicalize()?;
    for part in suffix.into_iter().rev() {
        resolved.push(part);
    }
    Ok(resolved)
}
