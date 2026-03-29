use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::Result;

/// Split a file into chunks of specified size
pub fn split_file(path: &Path, chunk_size: u64, output_dir: &Path) -> Result<Vec<PathBuf>> {
    let file = File::open(path)?;
    let file_size = file.metadata()?.len();
    let mut reader = BufReader::new(file);

    let filename = path.file_name().unwrap_or_default().to_string_lossy();

    let mut parts = Vec::new();
    let mut part_num = 1u32;
    let mut remaining = file_size;

    std::fs::create_dir_all(output_dir)?;

    while remaining > 0 {
        let part_path = output_dir.join(format!("{}.{:03}", filename, part_num));
        let mut part_file = BufWriter::new(File::create(&part_path)?);

        let to_write = std::cmp::min(remaining, chunk_size);
        let mut written = 0u64;
        let mut buf = [0u8; 65536];

        while written < to_write {
            let to_read = std::cmp::min((to_write - written) as usize, buf.len());
            let n = reader.read(&mut buf[..to_read])?;
            if n == 0 {
                break;
            }
            part_file.write_all(&buf[..n])?;
            written += n as u64;
        }

        parts.push(part_path);
        remaining -= written;
        part_num += 1;
    }

    Ok(parts)
}

/// Merge split parts back into a single file
pub fn merge_files(parts: &[PathBuf], output: &Path) -> Result<()> {
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut writer = BufWriter::new(File::create(output)?);

    for part in parts {
        let mut reader = BufReader::new(File::open(part)?);
        std::io::copy(&mut reader, &mut writer)?;
    }

    Ok(())
}
