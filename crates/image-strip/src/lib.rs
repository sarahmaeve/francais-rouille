pub mod exif;
pub mod jpeg;
pub mod png;

use std::fmt;
use std::path::Path;

use img_parts::jpeg::Jpeg;
use img_parts::png::Png;
use img_parts::ImageEXIF;

/// Supported image formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Jpeg,
    Png,
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImageFormat::Jpeg => write!(f, "JPEG"),
            ImageFormat::Png => write!(f, "PNG"),
        }
    }
}

/// Options controlling what metadata is preserved.
#[derive(Debug, Clone, Default)]
pub struct StripOptions {
    /// Keep ICC color profile data (APP2 in JPEG, iCCP in PNG).
    pub keep_icc: bool,
}

/// Report of a single file strip operation.
#[derive(Debug, Clone)]
pub struct StripReport {
    pub path: std::path::PathBuf,
    pub format: ImageFormat,
    pub segments_removed: Vec<String>,
    pub bytes_before: u64,
    pub bytes_after: u64,
}

impl fmt::Display for StripReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let saved = self.bytes_before.saturating_sub(self.bytes_after);
        if self.segments_removed.is_empty() {
            write!(
                f,
                "{}: {} — already clean",
                self.path.display(),
                self.format,
            )
        } else {
            write!(
                f,
                "{}: {} — removed {} segment(s) [{}], saved {} bytes ({} -> {})",
                self.path.display(),
                self.format,
                self.segments_removed.len(),
                self.segments_removed.join(", "),
                saved,
                self.bytes_before,
                self.bytes_after,
            )
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupported or unrecognized image format")]
    UnsupportedFormat,
    #[error("failed to parse {format} image: {reason}")]
    Parse { format: &'static str, reason: String },
}

/// Detect format from file extension.
pub fn detect_format(path: &Path) -> Option<ImageFormat> {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => Some(ImageFormat::Jpeg),
        Some("png") => Some(ImageFormat::Png),
        _ => None,
    }
}

/// Strip metadata from raw bytes of a known format.
///
/// Returns `(cleaned_bytes, removed_segment_names)`.
pub fn strip_metadata_bytes(
    data: &[u8],
    format: ImageFormat,
    opts: &StripOptions,
) -> Result<(Vec<u8>, Vec<String>), StripError> {
    match format {
        ImageFormat::Jpeg => {
            let mut img = Jpeg::from_bytes(data.to_vec().into()).map_err(|e| StripError::Parse {
                format: "JPEG",
                reason: e.to_string(),
            })?;
            // img-parts tracks EXIF (APP1) separately from the segment list.
            // Extract orientation before stripping so we can preserve it.
            let orientation = img
                .exif()
                .as_deref()
                .and_then(exif::read_orientation);
            let exif_bytes = img.exif().map(|b| b.to_vec());
            img.set_exif(None);
            let (mut bytes, mut removed) = jpeg::strip(&img, opts.keep_icc);

            // Determine if the original EXIF contained more than just orientation.
            let minimal_orient_exif = orientation
                .filter(|&o| o != 1)
                .map(exif::build_orientation_exif);
            let exif_was_stripped = match (&exif_bytes, &minimal_orient_exif) {
                (Some(original), Some(minimal)) => original.as_slice() != minimal.as_slice(),
                (Some(_), None) => true,
                (None, _) => false,
            };
            if exif_was_stripped {
                removed.insert(0, "APP1/EXIF+XMP".into());
            }

            // Write back a minimal EXIF with only the orientation tag.
            if let Some(orient_exif) = minimal_orient_exif {
                let mut img2 = Jpeg::from_bytes(bytes.into()).map_err(|e| {
                    StripError::Parse {
                        format: "JPEG",
                        reason: e.to_string(),
                    }
                })?;
                img2.set_exif(Some(orient_exif.into()));
                bytes = Vec::new();
                img2.encoder()
                    .write_to(&mut bytes)
                    .expect("encoding to Vec<u8> should not fail");
            }
            Ok((bytes, removed))
        }
        ImageFormat::Png => {
            let mut img = Png::from_bytes(data.to_vec().into()).map_err(|e| StripError::Parse {
                format: "PNG",
                reason: e.to_string(),
            })?;
            // img-parts tracks eXIf separately from the chunk list.
            let had_exif = img.exif().is_some();
            img.set_exif(None);
            let (bytes, mut removed) = png::strip(&img, opts.keep_icc);
            if had_exif {
                removed.insert(0, "eXIf".into());
            }
            Ok((bytes, removed))
        }
    }
}

/// Strip metadata from a file on disk.
pub fn strip_metadata(
    input: &Path,
    output: &Path,
    opts: &StripOptions,
) -> Result<StripReport, StripError> {
    let format = detect_format(input).ok_or(StripError::UnsupportedFormat)?;
    let data = std::fs::read(input)?;
    let bytes_before = data.len() as u64;

    let (cleaned, segments_removed) = strip_metadata_bytes(&data, format, opts)?;
    let bytes_after = cleaned.len() as u64;

    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, &cleaned)?;

    Ok(StripReport {
        path: input.to_path_buf(),
        format,
        segments_removed,
        bytes_before,
        bytes_after,
    })
}
