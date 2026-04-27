//! # archiver
//!
//! Universal archive library supporting all major compression formats.
//!
//! ## Supported Formats
//! - ZIP (read/write/password)
//! - TAR, TAR.GZ, TAR.BZ2, TAR.XZ, TAR.ZST (read/write)
//! - 7Z (read/write/password)
//! - RAR (read-only, with password)
//! - GZ, BZ2, XZ, ZST (single-file compression)
//!
//! ## Usage
//! ```no_run
//! use std::path::PathBuf;
//! use archiver::types::{OpenOptions, CreateOptions};
//!
//! // List archive contents
//! let entries = archiver::list("archive.zip", None).unwrap();
//!
//! // Extract all
//! archiver::extract_all("archive.tar.gz", "./output", None).unwrap();
//!
//! // Create with password
//! let opts = CreateOptions { password: Some("secret".into()), ..Default::default() };
//! archiver::create("secure.zip", &[PathBuf::from("file.txt")], Some(&opts)).unwrap();
//! ```

pub mod detect;
pub mod error;
mod formats;
pub mod split;
pub mod types;

use std::path::{Path, PathBuf};

use error::Result;
use formats::ArchiveHandler;
use types::{ArchiveFormat, CreateOptions, EntryInfo, OpenOptions};

fn get_handler(format: ArchiveFormat) -> Box<dyn ArchiveHandler> {
    match format {
        ArchiveFormat::Zip => Box::new(formats::zip_handler::ZipHandler),
        ArchiveFormat::Tar
        | ArchiveFormat::TarGz
        | ArchiveFormat::TarBz2
        | ArchiveFormat::TarXz
        | ArchiveFormat::TarZst => Box::new(formats::tar_handler::TarHandler::new(format)),
        ArchiveFormat::SevenZ => Box::new(formats::sevenz_handler::SevenZHandler),
        ArchiveFormat::Rar => Box::new(formats::rar_handler::RarHandler),
        ArchiveFormat::Gz | ArchiveFormat::Bz2 | ArchiveFormat::Xz | ArchiveFormat::Zst => {
            Box::new(formats::single_file::SingleFileHandler::new(format))
        }
    }
}

/// Detect the archive format from file extension
pub fn detect(path: impl AsRef<Path>) -> Result<ArchiveFormat> {
    detect::detect_format(path.as_ref())
}

/// List all entries in an archive
pub fn list(path: impl AsRef<Path>, opts: Option<&OpenOptions>) -> Result<Vec<EntryInfo>> {
    let path = path.as_ref();
    let format = detect::detect_format(path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.list(path, password)
}

/// Extract all files from an archive
pub fn extract_all(path: impl AsRef<Path>, dest: impl AsRef<Path>, opts: Option<&OpenOptions>) -> Result<()> {
    let path = path.as_ref();
    let format = detect::detect_format(path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.extract_all(path, dest.as_ref(), password)
}

/// Extract a single file from an archive
pub fn extract_file(
    path: impl AsRef<Path>,
    entry: &str,
    dest: impl AsRef<Path>,
    opts: Option<&OpenOptions>,
) -> Result<()> {
    let path = path.as_ref();
    let format = detect::detect_format(path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.extract_file(path, entry, dest.as_ref(), password)
}

/// Preview (read into memory) a single file from an archive
pub fn preview(path: impl AsRef<Path>, entry: &str, opts: Option<&OpenOptions>) -> Result<Vec<u8>> {
    let path = path.as_ref();
    let format = detect::detect_format(path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.preview(path, entry, password)
}

/// Create a new archive from source files/directories
pub fn create(archive_path: impl AsRef<Path>, sources: &[PathBuf], opts: Option<&CreateOptions>) -> Result<()> {
    let archive_path = archive_path.as_ref();
    let format = detect::detect_format(archive_path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.create(archive_path, sources, password)
}

/// Add files to an existing archive
pub fn add(archive_path: impl AsRef<Path>, sources: &[PathBuf], opts: Option<&CreateOptions>) -> Result<()> {
    let archive_path = archive_path.as_ref();
    let format = detect::detect_format(archive_path)?;
    let handler = get_handler(format);
    let password = opts.and_then(|o| o.password.as_deref());
    handler.add(archive_path, sources, password)
}

/// Split an archive file into parts of specified size
pub fn split_archive(path: impl AsRef<Path>, chunk_size: u64, output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    split::split_file(path.as_ref(), chunk_size, output_dir.as_ref())
}

/// Merge split archive parts back into a single file
pub fn merge_parts(parts: &[PathBuf], output: impl AsRef<Path>) -> Result<()> {
    split::merge_files(parts, output.as_ref())
}
