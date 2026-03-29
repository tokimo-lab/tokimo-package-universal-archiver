use std::fs::{self, File};
use std::io::{BufReader, Cursor, Read, Write};
use std::path::{Path, PathBuf};
use tar::{Archive as TarArchive, Builder as TarBuilder};
use walkdir::WalkDir;

use crate::error::{ArchiveError, Result};
use crate::types::{ArchiveFormat, EntryInfo};

use super::ArchiveHandler;

pub struct TarHandler {
    pub format: ArchiveFormat,
}

impl TarHandler {
    pub fn new(format: ArchiveFormat) -> Self {
        Self { format }
    }

    fn decompress_to_tar(&self, path: &Path) -> Result<Vec<u8>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut data = Vec::new();

        match self.format {
            ArchiveFormat::Tar => {
                let mut f = File::open(path)?;
                f.read_to_end(&mut data)?;
            }
            ArchiveFormat::TarGz => {
                let mut decoder = flate2::read::GzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::TarBz2 => {
                let mut decoder = bzip2::read::BzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::TarXz => {
                let mut decoder = xz2::read::XzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::TarZst => {
                let mut decoder =
                    zstd::Decoder::new(reader).map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                decoder.read_to_end(&mut data)?;
            }
            _ => {
                return Err(ArchiveError::UnsupportedFormat(format!(
                    "{}",
                    self.format
                )))
            }
        }

        Ok(data)
    }

    fn compress_tar(&self, tar_data: &[u8], output: &Path) -> Result<()> {
        match self.format {
            ArchiveFormat::Tar => {
                fs::write(output, tar_data)?;
            }
            ArchiveFormat::TarGz => {
                let file = File::create(output)?;
                let mut encoder =
                    flate2::write::GzEncoder::new(file, flate2::Compression::default());
                encoder.write_all(tar_data)?;
                encoder.finish()?;
            }
            ArchiveFormat::TarBz2 => {
                let file = File::create(output)?;
                let mut encoder =
                    bzip2::write::BzEncoder::new(file, bzip2::Compression::default());
                encoder.write_all(tar_data)?;
                encoder.finish()?;
            }
            ArchiveFormat::TarXz => {
                let file = File::create(output)?;
                let mut encoder = xz2::write::XzEncoder::new(file, 6);
                encoder.write_all(tar_data)?;
                encoder.finish()?;
            }
            ArchiveFormat::TarZst => {
                let file = File::create(output)?;
                let mut encoder = zstd::Encoder::new(file, 3)
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                encoder.write_all(tar_data)?;
                encoder.finish()?;
            }
            _ => {
                return Err(ArchiveError::UnsupportedFormat(format!(
                    "{}",
                    self.format
                )))
            }
        }
        Ok(())
    }

    fn collect_sources(sources: &[PathBuf]) -> Vec<(PathBuf, String)> {
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
                    files.push((path, rel));
                }
            } else {
                let name = source
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                files.push((source.clone(), name));
            }
        }
        files
    }
}

impl ArchiveHandler for TarHandler {
    fn list(&self, path: &Path, _password: Option<&str>) -> Result<Vec<EntryInfo>> {
        let data = self.decompress_to_tar(path)?;
        let mut archive = TarArchive::new(Cursor::new(data));
        let mut entries = Vec::new();

        for entry in archive
            .entries()
            .map_err(|e| ArchiveError::FormatError(e.to_string()))?
        {
            let entry = entry.map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            let header = entry.header();
            let path_str = entry
                .path()
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?
                .to_string_lossy()
                .to_string();

            entries.push(EntryInfo {
                path: path_str,
                size: header.size().unwrap_or(0),
                compressed_size: None,
                is_dir: header.entry_type().is_dir(),
                modified: header.mtime().ok().map(|t| {
                    chrono::DateTime::from_timestamp(t as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                }),
                encrypted: false,
            });
        }

        Ok(entries)
    }

    fn extract_all(&self, path: &Path, dest: &Path, _password: Option<&str>) -> Result<()> {
        let data = self.decompress_to_tar(path)?;
        let mut archive = TarArchive::new(Cursor::new(data));
        fs::create_dir_all(dest)?;
        archive
            .unpack(dest)
            .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        Ok(())
    }

    fn extract_file(
        &self,
        path: &Path,
        entry_name: &str,
        dest: &Path,
        password: Option<&str>,
    ) -> Result<()> {
        let data = self.preview(path, entry_name, password)?;
        let out_path = dest.join(entry_name);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, data)?;
        Ok(())
    }

    fn preview(&self, path: &Path, entry_name: &str, _password: Option<&str>) -> Result<Vec<u8>> {
        let data = self.decompress_to_tar(path)?;
        let mut archive = TarArchive::new(Cursor::new(data));

        for entry in archive
            .entries()
            .map_err(|e| ArchiveError::FormatError(e.to_string()))?
        {
            let mut entry = entry.map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            let entry_path = entry
                .path()
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?
                .to_string_lossy()
                .to_string();

            let name_a = entry_path.trim_end_matches('/');
            let name_b = entry_name.trim_end_matches('/');
            if name_a == name_b {
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                return Ok(buf);
            }
        }

        Err(ArchiveError::EntryNotFound(entry_name.to_string()))
    }

    fn create(
        &self,
        archive_path: &Path,
        sources: &[PathBuf],
        _password: Option<&str>,
    ) -> Result<()> {
        let mut tar_data = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_data);
            let files = Self::collect_sources(sources);

            for (file_path, archive_name) in &files {
                if file_path.is_dir() {
                    builder
                        .append_dir(archive_name, file_path)
                        .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                } else if file_path.is_file() {
                    let mut f = File::open(file_path)?;
                    builder
                        .append_file(archive_name, &mut f)
                        .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                }
            }

            builder
                .finish()
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        }

        self.compress_tar(&tar_data, archive_path)?;
        Ok(())
    }

    fn add(
        &self,
        archive_path: &Path,
        sources: &[PathBuf],
        _password: Option<&str>,
    ) -> Result<()> {
        // Read existing tar data
        let existing_data = self.decompress_to_tar(archive_path)?;

        let mut tar_data = Vec::new();
        {
            let mut builder = TarBuilder::new(&mut tar_data);

            // Read and re-add existing entries
            let mut old_archive = TarArchive::new(Cursor::new(existing_data));
            let mut existing_entries: Vec<(tar::Header, String, Vec<u8>)> = Vec::new();

            for entry in old_archive
                .entries()
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?
            {
                let mut entry = entry.map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                let header = entry.header().clone();
                let path_str = entry
                    .path()
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?
                    .to_string_lossy()
                    .to_string();
                let mut data = Vec::new();
                entry.read_to_end(&mut data)?;
                existing_entries.push((header, path_str, data));
            }

            for (mut header, path_str, data) in existing_entries {
                builder
                    .append_data(&mut header, &path_str, Cursor::new(data))
                    .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
            }

            // Add new entries
            let files = Self::collect_sources(sources);
            for (file_path, archive_name) in &files {
                if file_path.is_dir() {
                    builder
                        .append_dir(archive_name, file_path)
                        .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                } else if file_path.is_file() {
                    let mut f = File::open(file_path)?;
                    builder
                        .append_file(archive_name, &mut f)
                        .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                }
            }

            builder
                .finish()
                .map_err(|e| ArchiveError::FormatError(e.to_string()))?;
        }

        self.compress_tar(&tar_data, archive_path)?;
        Ok(())
    }

    fn supports_password(&self) -> bool {
        false
    }
}
