use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{ArchiveError, Result};
use crate::types::{ArchiveFormat, EntryInfo};

use super::ArchiveHandler;

pub struct SingleFileHandler {
    pub format: ArchiveFormat,
}

impl SingleFileHandler {
    pub fn new(format: ArchiveFormat) -> Self {
        Self { format }
    }

    fn inner_name(path: &Path) -> String {
        path.file_stem().unwrap_or_default().to_string_lossy().to_string()
    }

    fn decompress(&self, path: &Path) -> Result<Vec<u8>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut data = Vec::new();

        match self.format {
            ArchiveFormat::Gz => {
                let mut decoder = flate2::read::GzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::Bz2 => {
                let mut decoder = bzip2::read::BzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::Xz => {
                let mut decoder = xz2::read::XzDecoder::new(reader);
                decoder.read_to_end(&mut data)?;
            }
            ArchiveFormat::Zst => {
                let mut decoder = zstd::Decoder::new(reader).map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                decoder.read_to_end(&mut data)?;
            }
            _ => return Err(ArchiveError::UnsupportedFormat(format!("{}", self.format))),
        }

        Ok(data)
    }

    fn compress(&self, data: &[u8], path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);

        match self.format {
            ArchiveFormat::Gz => {
                let mut encoder = flate2::write::GzEncoder::new(writer, flate2::Compression::default());
                encoder.write_all(data)?;
                encoder.finish()?;
            }
            ArchiveFormat::Bz2 => {
                let mut encoder = bzip2::write::BzEncoder::new(writer, bzip2::Compression::default());
                encoder.write_all(data)?;
                encoder.finish()?;
            }
            ArchiveFormat::Xz => {
                let mut encoder = xz2::write::XzEncoder::new(writer, 6);
                encoder.write_all(data)?;
                encoder.finish()?;
            }
            ArchiveFormat::Zst => {
                let mut encoder =
                    zstd::Encoder::new(writer, 3).map_err(|e| ArchiveError::FormatError(e.to_string()))?;
                encoder.write_all(data)?;
                encoder.finish()?;
            }
            _ => return Err(ArchiveError::UnsupportedFormat(format!("{}", self.format))),
        }

        Ok(())
    }
}

impl ArchiveHandler for SingleFileHandler {
    fn list(&self, path: &Path, _password: Option<&str>) -> Result<Vec<EntryInfo>> {
        let metadata = fs::metadata(path)?;
        let decompressed = self.decompress(path)?;
        Ok(vec![EntryInfo {
            path: Self::inner_name(path),
            size: decompressed.len() as u64,
            compressed_size: Some(metadata.len()),
            is_dir: false,
            modified: None,
            encrypted: false,
        }])
    }

    fn extract_all(&self, path: &Path, dest: &Path, _password: Option<&str>) -> Result<()> {
        let data = self.decompress(path)?;
        fs::create_dir_all(dest)?;
        let out_path = dest.join(Self::inner_name(path));
        fs::write(&out_path, data)?;
        Ok(())
    }

    fn extract_file(&self, path: &Path, _entry: &str, dest: &Path, password: Option<&str>) -> Result<()> {
        self.extract_all(path, dest, password)
    }

    fn preview(&self, path: &Path, _entry: &str, _password: Option<&str>) -> Result<Vec<u8>> {
        self.decompress(path)
    }

    fn create(&self, archive_path: &Path, sources: &[PathBuf], _password: Option<&str>) -> Result<()> {
        if sources.len() != 1 || sources[0].is_dir() {
            return Err(ArchiveError::NotSupported(
                "Single-file compression only supports one file".to_string(),
            ));
        }

        let data = fs::read(&sources[0])?;
        self.compress(&data, archive_path)?;
        Ok(())
    }

    fn add(&self, _archive_path: &Path, _sources: &[PathBuf], _password: Option<&str>) -> Result<()> {
        Err(ArchiveError::NotSupported(
            "Cannot add files to single-file compression format".to_string(),
        ))
    }

    fn supports_password(&self) -> bool {
        false
    }
}
