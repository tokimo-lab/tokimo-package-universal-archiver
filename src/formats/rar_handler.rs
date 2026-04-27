use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{ArchiveError, Result};
use crate::types::EntryInfo;

use super::ArchiveHandler;

pub struct RarHandler;

impl ArchiveHandler for RarHandler {
    fn list(&self, path: &Path, password: Option<&str>) -> Result<Vec<EntryInfo>> {
        let path_str = path.to_string_lossy().to_string();
        let archive = if let Some(pw) = password {
            unrar::Archive::with_password(&path_str, pw)
        } else {
            unrar::Archive::new(&path_str)
        };

        let opened = archive
            .open_for_listing()
            .map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?;

        let mut entries = Vec::new();
        for entry in opened {
            let entry = entry.map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?;
            entries.push(EntryInfo {
                path: entry.filename.to_string_lossy().to_string(),
                size: entry.unpacked_size,
                compressed_size: None,
                is_dir: entry.is_directory(),
                modified: None,
                encrypted: entry.is_encrypted(),
            });
        }

        Ok(entries)
    }

    fn extract_all(&self, path: &Path, dest: &Path, password: Option<&str>) -> Result<()> {
        fs::create_dir_all(dest)?;
        let path_str = path.to_string_lossy().to_string();

        let archive = if let Some(pw) = password {
            unrar::Archive::with_password(&path_str, pw)
        } else {
            unrar::Archive::new(&path_str)
        };

        let opened = archive
            .open_for_listing()
            .map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?;

        // Get list of files first
        let _file_list: Vec<String> = opened
            .filter_map(|e| e.ok())
            .map(|e| e.filename.to_string_lossy().to_string())
            .collect();

        // Re-open for processing
        let archive2 = if let Some(pw) = password {
            unrar::Archive::with_password(&path_str, pw)
        } else {
            unrar::Archive::new(&path_str)
        };

        let _dest_str = dest.to_string_lossy().to_string();
        let mut cursor = archive2
            .open_for_processing()
            .map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?;

        while let Ok(Some(header)) = cursor.read_header() {
            cursor = if header.entry().is_file() {
                header
                    .extract_with_base(dest)
                    .map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?
            } else {
                // Create directory
                let dir_path = dest.join(header.entry().filename.clone());
                fs::create_dir_all(&dir_path).ok();
                header
                    .skip()
                    .map_err(|e| ArchiveError::FormatError(format!("{:?}", e)))?
            };
        }

        Ok(())
    }

    fn extract_file(&self, path: &Path, entry_name: &str, dest: &Path, password: Option<&str>) -> Result<()> {
        let data = self.preview(path, entry_name, password)?;
        let out_path = dest.join(entry_name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, data)?;
        Ok(())
    }

    fn preview(&self, path: &Path, entry_name: &str, password: Option<&str>) -> Result<Vec<u8>> {
        // Extract to a temp dir and read the file
        let temp_dir = tempfile::tempdir()?;
        self.extract_all(path, temp_dir.path(), password)?;

        let file_path = temp_dir.path().join(entry_name);
        if file_path.exists() {
            let mut data = Vec::new();
            File::open(&file_path)?.read_to_end(&mut data)?;
            Ok(data)
        } else {
            Err(ArchiveError::EntryNotFound(entry_name.to_string()))
        }
    }

    fn create(&self, _archive_path: &Path, _sources: &[PathBuf], _password: Option<&str>) -> Result<()> {
        Err(ArchiveError::NotSupported(
            "RAR creation is not supported (proprietary format). Use 7z or zip instead.".to_string(),
        ))
    }

    fn add(&self, _archive_path: &Path, _sources: &[PathBuf], _password: Option<&str>) -> Result<()> {
        Err(ArchiveError::NotSupported(
            "RAR modification is not supported (proprietary format). Use 7z or zip instead.".to_string(),
        ))
    }

    fn supports_password(&self) -> bool {
        true
    }
}
