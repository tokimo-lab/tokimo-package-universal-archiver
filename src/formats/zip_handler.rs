use std::fs::{self, File};
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::CompressionMethod;
use zip::read::ZipArchive;
use zip::unstable::write::FileOptionsExt;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::error::{ArchiveError, Result};
use crate::types::EntryInfo;

use super::ArchiveHandler;

pub struct ZipHandler;

impl ZipHandler {
    fn collect_sources(sources: &[PathBuf]) -> Result<Vec<(PathBuf, String)>> {
        let mut files = Vec::new();
        for source in sources {
            if source.is_dir() {
                let base = source.parent().unwrap_or(source);
                for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                    let path = entry.path().to_path_buf();
                    let rel = path
                        .strip_prefix(base)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .replace('\\', "/");
                    if entry.file_type().is_dir() {
                        if !rel.is_empty() {
                            files.push((path, format!("{}/", rel)));
                        }
                    } else {
                        files.push((path, rel));
                    }
                }
            } else {
                let name = source.file_name().unwrap_or_default().to_string_lossy().to_string();
                files.push((source.clone(), name));
            }
        }
        Ok(files)
    }
}

impl ArchiveHandler for ZipHandler {
    fn list(&self, path: &Path, _password: Option<&str>) -> Result<Vec<EntryInfo>> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        let mut entries = Vec::new();

        for i in 0..archive.len() {
            let entry = archive
                .by_index_raw(i)
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            entries.push(EntryInfo {
                path: entry.name().to_string(),
                size: entry.size(),
                compressed_size: Some(entry.compressed_size()),
                is_dir: entry.is_dir(),
                modified: None,
                encrypted: entry.encrypted(),
            });
        }

        Ok(entries)
    }

    fn extract_all(&self, path: &Path, dest: &Path, password: Option<&str>) -> Result<()> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        fs::create_dir_all(dest)?;

        for i in 0..archive.len() {
            let mut entry = if let Some(pw) = password {
                archive
                    .by_index_decrypt(i, pw.as_bytes())
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?
            } else {
                archive
                    .by_index(i)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?
            };

            let out_path = dest.join(entry.mangled_name());

            if entry.is_dir() {
                fs::create_dir_all(&out_path)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut out_file = File::create(&out_path)?;
                std::io::copy(&mut entry, &mut out_file)?;
            }
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
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file).map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        let mut entry = if let Some(pw) = password {
            archive
                .by_name_decrypt(entry_name, pw.as_bytes())
                .map_err(|e| ArchiveError::EntryNotFound(e.to_string()))?
        } else {
            archive
                .by_name(entry_name)
                .map_err(|e| ArchiveError::EntryNotFound(e.to_string()))?
        };

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        Ok(buf)
    }

    fn create(&self, archive_path: &Path, sources: &[PathBuf], password: Option<&str>) -> Result<()> {
        let file = File::create(archive_path)?;
        let mut zip = ZipWriter::new(file);

        let options = if let Some(pw) = password {
            SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .with_deprecated_encryption(pw.as_bytes())
        } else {
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
        };

        let files = Self::collect_sources(sources)?;

        for (file_path, archive_name) in &files {
            if file_path.is_dir() {
                zip.add_directory(archive_name, options)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            } else {
                zip.start_file(archive_name, options)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                let mut f = File::open(file_path)?;
                std::io::copy(&mut f, &mut zip)?;
            }
        }

        zip.finish().map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        Ok(())
    }

    fn add(&self, archive_path: &Path, sources: &[PathBuf], password: Option<&str>) -> Result<()> {
        let old_data = fs::read(archive_path)?;
        let mut old_archive =
            ZipArchive::new(Cursor::new(&old_data)).map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        let file = File::create(archive_path)?;
        let mut zip = ZipWriter::new(file);

        // Copy existing entries using raw copy
        for i in 0..old_archive.len() {
            let entry = old_archive
                .by_index_raw(i)
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            zip.raw_copy_file(entry)
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        }

        // Add new entries
        let options = if let Some(pw) = password {
            SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .with_deprecated_encryption(pw.as_bytes())
        } else {
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated)
        };

        let files = Self::collect_sources(sources)?;
        for (file_path, archive_name) in &files {
            if file_path.is_dir() {
                zip.add_directory(archive_name, options)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            } else {
                zip.start_file(archive_name, options)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                let mut f = File::open(file_path)?;
                std::io::copy(&mut f, &mut zip)?;
            }
        }

        zip.finish().map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        Ok(())
    }

    fn supports_password(&self) -> bool {
        true
    }
}
