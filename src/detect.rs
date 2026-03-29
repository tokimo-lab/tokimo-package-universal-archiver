use std::path::Path;

use crate::error::{ArchiveError, Result};
use crate::types::ArchiveFormat;

/// Detect archive format from file extension
pub fn detect_format(path: &Path) -> Result<ArchiveFormat> {
    let name = path.to_string_lossy().to_lowercase();

    // Check compound extensions first
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        return Ok(ArchiveFormat::TarGz);
    }
    if name.ends_with(".tar.bz2") || name.ends_with(".tbz2") {
        return Ok(ArchiveFormat::TarBz2);
    }
    if name.ends_with(".tar.xz") || name.ends_with(".txz") {
        return Ok(ArchiveFormat::TarXz);
    }
    if name.ends_with(".tar.zst") || name.ends_with(".tzst") {
        return Ok(ArchiveFormat::TarZst);
    }

    // Simple extensions
    if name.ends_with(".zip") {
        return Ok(ArchiveFormat::Zip);
    }
    if name.ends_with(".tar") {
        return Ok(ArchiveFormat::Tar);
    }
    if name.ends_with(".7z") {
        return Ok(ArchiveFormat::SevenZ);
    }
    if name.ends_with(".rar") {
        return Ok(ArchiveFormat::Rar);
    }
    if name.ends_with(".gz") {
        return Ok(ArchiveFormat::Gz);
    }
    if name.ends_with(".bz2") {
        return Ok(ArchiveFormat::Bz2);
    }
    if name.ends_with(".xz") {
        return Ok(ArchiveFormat::Xz);
    }
    if name.ends_with(".zst") {
        return Ok(ArchiveFormat::Zst);
    }

    Err(ArchiveError::UnsupportedFormat(format!(
        "Cannot detect format from path: {}",
        path.display()
    )))
}
