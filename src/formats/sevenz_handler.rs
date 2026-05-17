use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::{ArchiveError, Result};
use crate::types::EntryInfo;

use super::ArchiveHandler;

pub struct SevenZHandler;

fn make_entry(name: String, is_dir: bool) -> sevenz_rust2::ArchiveEntry {
    sevenz_rust2::ArchiveEntry {
        name,
        is_directory: is_dir,
        ..Default::default()
    }
}

impl ArchiveHandler for SevenZHandler {
    fn list(&self, path: &Path, password: Option<&str>) -> Result<Vec<EntryInfo>> {
        let pw = password
            .map(sevenz_rust2::Password::from)
            .unwrap_or_else(sevenz_rust2::Password::empty);

        let sz = sevenz_rust2::ArchiveReader::open(path, pw).map_err(|e| ArchiveError::FormatError(format!("{}", e)))?;

        let mut entries = Vec::new();
        for entry in sz.archive().files.iter() {
            entries.push(EntryInfo {
                path: entry.name().to_string(),
                size: entry.size(),
                compressed_size: None,
                is_dir: entry.is_directory(),
                modified: None,
                encrypted: false,
            });
        }

        Ok(entries)
    }

    fn extract_all(&self, path: &Path, dest: &Path, password: Option<&str>) -> Result<()> {
        fs::create_dir_all(dest)?;

        let pw = password
            .map(sevenz_rust2::Password::from)
            .unwrap_or_else(sevenz_rust2::Password::empty);

        let mut sz =
            sevenz_rust2::ArchiveReader::open(path, pw).map_err(|e| ArchiveError::FormatError(format!("{}", e)))?;

        sz.for_each_entries(|entry, reader| {
            let dest_path = dest.join(entry.name());
            if entry.is_directory() {
                fs::create_dir_all(&dest_path).ok();
            } else {
                if let Some(parent) = dest_path.parent() {
                    fs::create_dir_all(parent).ok();
                }
                let mut file = fs::File::create(&dest_path)?;
                std::io::copy(reader, &mut file)?;
            }
            Ok(true)
        })
        .map_err(|e| ArchiveError::FormatError(format!("{}", e)))?;

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
        let pw = password
            .map(sevenz_rust2::Password::from)
            .unwrap_or_else(sevenz_rust2::Password::empty);

        let mut sz =
            sevenz_rust2::ArchiveReader::open(path, pw).map_err(|e| ArchiveError::FormatError(format!("{}", e)))?;

        let mut found_data: Option<Vec<u8>> = None;
        let target = entry_name.to_string();

        sz.for_each_entries(|entry, reader| {
            if entry.name() == target {
                let mut buf = Vec::new();
                reader.read_to_end(&mut buf).ok();
                found_data = Some(buf);
            } else {
                std::io::copy(reader, &mut std::io::sink()).ok();
            }
            Ok(true)
        })
        .map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        found_data.ok_or_else(|| ArchiveError::EntryNotFound(entry_name.to_string()))
    }

    fn create(&self, archive_path: &Path, sources: &[PathBuf], _password: Option<&str>) -> Result<()> {
        let mut sz =
            sevenz_rust2::ArchiveWriter::create(archive_path).map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        for source in sources {
            if source.is_dir() {
                let base = source.parent().unwrap_or(source);
                for entry in WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
                    let fpath = entry.path();
                    let rel = fpath
                        .strip_prefix(base)
                        .unwrap_or(fpath)
                        .to_string_lossy()
                        .replace('\\', "/");

                    if entry.file_type().is_dir() {
                        if !rel.is_empty() {
                            sz.push_archive_entry::<&[u8]>(make_entry(format!("{}/", rel), true), None)
                                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                        }
                    } else {
                        let data = fs::read(fpath)?;
                        sz.push_archive_entry(make_entry(rel, false), Some(data.as_slice()))
                            .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                    }
                }
            } else {
                let name = source.file_name().unwrap_or_default().to_string_lossy().to_string();
                let data = fs::read(source)?;
                sz.push_archive_entry(make_entry(name, false), Some(data.as_slice()))
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            }
        }

        sz.finish().map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        Ok(())
    }

    fn add(&self, archive_path: &Path, sources: &[PathBuf], password: Option<&str>) -> Result<()> {
        // Extract to temp dir, add new files, recreate
        let temp_dir = tempfile::tempdir()?;
        self.extract_all(archive_path, temp_dir.path(), password)?;

        // Copy new sources into temp dir
        for source in sources {
            if source.is_dir() {
                let dest = temp_dir.path().join(source.file_name().unwrap_or_default());
                copy_dir_recursive(source, &dest)?;
            } else {
                let dest = temp_dir.path().join(source.file_name().unwrap_or_default());
                fs::copy(source, &dest)?;
            }
        }

        // Collect all items in temp_dir as sources
        let items: Vec<PathBuf> = fs::read_dir(temp_dir.path())?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .collect();

        // Recreate archive
        let mut sz =
            sevenz_rust2::ArchiveWriter::create(archive_path).map_err(|e| ArchiveError::FormatError(e.to_string()))?;

        for item in &items {
            if item.is_dir() {
                for entry in WalkDir::new(item).into_iter().filter_map(|e| e.ok()) {
                    let fpath = entry.path();
                    let rel = fpath
                        .strip_prefix(temp_dir.path())
                        .unwrap_or(fpath)
                        .to_string_lossy()
                        .replace('\\', "/");

                    if entry.file_type().is_dir() {
                        if !rel.is_empty() {
                            sz.push_archive_entry::<&[u8]>(make_entry(format!("{}/", rel), true), None)
                                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                        }
                    } else {
                        let data = fs::read(fpath)?;
                        sz.push_archive_entry(make_entry(rel, false), Some(data.as_slice()))
                            .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                    }
                }
            } else {
                let name = item
                    .strip_prefix(temp_dir.path())
                    .unwrap_or(item)
                    .to_string_lossy()
                    .replace('\\', "/");
                let data = fs::read(item)?;
                sz.push_archive_entry(make_entry(name, false), Some(data.as_slice()))
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            }
        }

        sz.finish().map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        Ok(())
    }

    fn supports_password(&self) -> bool {
        true
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}
