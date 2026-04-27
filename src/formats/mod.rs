pub mod rar_handler;
pub mod sevenz_handler;
pub mod single_file;
pub mod tar_handler;
pub mod zip_handler;

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::types::EntryInfo;

/// Internal trait for format-specific archive operations
pub(crate) trait ArchiveHandler {
    fn list(&self, path: &Path, password: Option<&str>) -> Result<Vec<EntryInfo>>;
    fn extract_all(&self, path: &Path, dest: &Path, password: Option<&str>) -> Result<()>;
    fn extract_file(&self, path: &Path, entry: &str, dest: &Path, password: Option<&str>) -> Result<()>;
    fn preview(&self, path: &Path, entry: &str, password: Option<&str>) -> Result<Vec<u8>>;
    fn create(&self, archive_path: &Path, sources: &[PathBuf], password: Option<&str>) -> Result<()>;
    fn add(&self, archive_path: &Path, sources: &[PathBuf], password: Option<&str>) -> Result<()>;
    #[allow(dead_code)]
    fn supports_password(&self) -> bool;
}
