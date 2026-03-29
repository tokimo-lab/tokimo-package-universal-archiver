# tokimo-universal-archiver

A universal archive library for Rust that supports **all major compression formats** with a unified API for listing, extracting, creating, previewing, adding files, password encryption, and split/merge operations.

## Supported Formats

| Format | Extension(s) | List | Preview | Extract All | Extract Single | Create | Add Files | Password | Split/Merge |
|--------|-------------|:----:|:-------:|:-----------:|:--------------:|:------:|:---------:|:--------:|:-----------:|
| **ZIP** | `.zip` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **TAR** | `.tar` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.GZ** | `.tar.gz` `.tgz` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.BZ2** | `.tar.bz2` `.tbz2` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.XZ** | `.tar.xz` `.txz` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.ZST** | `.tar.zst` `.tzst` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **7Z** | `.7z` | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **RAR** | `.rar` | ✅ | ✅ | ✅ | ✅ | ❌¹ | ❌¹ | ✅ | ✅ |
| **GZ** | `.gz` | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **BZ2** | `.bz2` | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **XZ** | `.xz` | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **ZST** | `.zst` | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |

> ¹ RAR is a proprietary format — creation/modification requires the official RAR tool. Read operations are fully supported.
>
> ² Single-file compression formats (GZ/BZ2/XZ/ZST) compress one file at a time, so "Add" is not applicable.
>
> — indicates the feature is not applicable for this format (not a limitation).

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
tokimo-universal-archiver = "0.1"
```

## Quick Start

```rust
use std::path::PathBuf;
use archiver::types::{OpenOptions, CreateOptions};

// List archive contents
let entries = archiver::list("archive.zip", None)?;
for entry in &entries {
    println!("{} ({} bytes)", entry.path, entry.size);
}

// Extract all files
archiver::extract_all("data.tar.gz", "./output", None)?;

// Extract a single file
archiver::extract_file("data.7z", "docs/readme.md", "./output", None)?;

// Preview file contents (read into memory)
let data = archiver::preview("archive.zip", "config.json", None)?;
let text = String::from_utf8(data)?;

// Create a new archive
archiver::create(
    "backup.tar.gz",
    &[PathBuf::from("src/"), PathBuf::from("Cargo.toml")],
    None,
)?;

// Create with password (ZIP / 7Z)
let opts = CreateOptions {
    password: Some("secret".into()),
    ..Default::default()
};
archiver::create("secure.zip", &[PathBuf::from("private/")], Some(&opts))?;

// Add files to existing archive
archiver::add("backup.tar.gz", &[PathBuf::from("new_file.txt")], None)?;

// Split large archive into parts
let parts = archiver::split_archive("huge.zip", 10_000_000, "./parts")?;

// Merge parts back
archiver::merge_parts(&parts, "restored.zip")?;
```

## API Reference

### Core Functions

| Function | Description |
|----------|-------------|
| `archiver::detect(path)` | Detect archive format from file extension |
| `archiver::list(path, opts)` | List all entries in an archive |
| `archiver::extract_all(path, dest, opts)` | Extract all files to a directory |
| `archiver::extract_file(path, entry, dest, opts)` | Extract a single file |
| `archiver::preview(path, entry, opts)` | Read a file's contents into memory |
| `archiver::create(path, sources, opts)` | Create a new archive from files/directories |
| `archiver::add(path, sources, opts)` | Add files to an existing archive |
| `archiver::split_archive(path, chunk_size, output_dir)` | Split an archive into fixed-size parts |
| `archiver::merge_parts(parts, output)` | Merge split parts back into one file |

### Types

```rust
// Entry information returned by list()
pub struct EntryInfo {
    pub path: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub encrypted: bool,
}

// Options for read operations
pub struct OpenOptions {
    pub password: Option<String>,
}

// Options for write operations
pub struct CreateOptions {
    pub password: Option<String>,
    pub compression_level: Option<i32>,
}

// Supported formats
pub enum ArchiveFormat {
    Zip, Tar, TarGz, TarBz2, TarXz, TarZst,
    SevenZ, Rar, Gz, Bz2, Xz, Zst,
}
```

### Error Handling

All functions return `Result<T, ArchiveError>`:

```rust
pub enum ArchiveError {
    Io(std::io::Error),
    UnsupportedFormat(String),
    PasswordRequired,
    InvalidPassword,
    EntryNotFound(String),
    FormatError(String),
    NotSupported(String),
    Other(String),
}
```

## CLI Tool

A CLI tool is included for testing and quick operations:

```bash
# Build
cargo build --release

# List archive contents
archiver-cli list archive.zip

# Extract with password
archiver-cli extract encrypted.7z -d ./output -p mypassword

# Create archive from directory
archiver-cli create backup.tar.gz ./src ./Cargo.toml

# Preview a file
archiver-cli preview archive.zip src/main.rs

# Split and merge
archiver-cli split huge.zip -s 10485760 -o ./parts
archiver-cli merge restored.zip ./parts/*.001 ./parts/*.002 ...
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`zip`](https://crates.io/crates/zip) v2 | ZIP read/write with AES encryption |
| [`tar`](https://crates.io/crates/tar) v0.4 | TAR archive handling |
| [`flate2`](https://crates.io/crates/flate2) v1 | Gzip compression |
| [`bzip2`](https://crates.io/crates/bzip2) v0.5 | Bzip2 compression |
| [`xz2`](https://crates.io/crates/xz2) v0.1 | XZ/LZMA compression |
| [`zstd`](https://crates.io/crates/zstd) v0.13 | Zstandard compression |
| [`sevenz-rust`](https://crates.io/crates/sevenz-rust) v0.6 | 7-Zip read/write |
| [`unrar`](https://crates.io/crates/unrar) v0.5 | RAR read (bundled C library) |

## Architecture

```
src/
├── lib.rs              # Public API (detect, list, extract, create, add, split, merge)
├── error.rs            # ArchiveError enum
├── types.rs            # EntryInfo, ArchiveFormat, OpenOptions, CreateOptions
├── detect.rs           # Format detection from file extension
├── split.rs            # File splitting and merging
└── formats/
    ├── mod.rs           # ArchiveHandler trait
    ├── zip_handler.rs   # ZIP implementation
    ├── tar_handler.rs   # TAR/TGZ/TBZ2/TXZ/TZST implementation
    ├── sevenz_handler.rs# 7Z implementation
    ├── rar_handler.rs   # RAR implementation (read-only)
    └── single_file.rs   # GZ/BZ2/XZ/ZST single-file compression
```

## License

MIT

---

# 中文文档

## tokimo-universal-archiver

一个通用的 Rust 压缩归档库，通过统一的 API 支持**所有主流压缩格式**的查看结构、预览、解压、创建、添加文件、密码加密和分包/合并操作。

### 功能支持矩阵

| 格式 | 查看结构 | 预览 | 全部解压 | 单个解压 | 创建 | 添加文件 | 密码 | 分包/合并 |
|------|:--------:|:----:|:--------:|:--------:|:----:|:--------:|:----:|:---------:|
| **ZIP** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **TAR** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.GZ** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.BZ2** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.XZ** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **TAR.ZST** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | — | ✅ |
| **7Z** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **RAR** | ✅ | ✅ | ✅ | ✅ | ❌¹ | ❌¹ | ✅ | ✅ |
| **GZ** | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **BZ2** | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **XZ** | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |
| **ZST** | ✅ | ✅ | ✅ | ✅ | ✅ | —² | — | ✅ |

> ¹ RAR 是专有格式，创建/修改需要官方 RAR 工具，读取操作完全支持。
>
> ² 单文件压缩格式（GZ/BZ2/XZ/ZST）一次只压缩一个文件，"添加"不适用。

### 快速上手

```rust
use std::path::PathBuf;
use archiver::types::{OpenOptions, CreateOptions};

// 查看归档内容
let entries = archiver::list("archive.zip", None)?;

// 解压全部文件
archiver::extract_all("data.tar.gz", "./output", None)?;

// 解压单个文件
archiver::extract_file("data.7z", "docs/readme.md", "./output", None)?;

// 预览文件内容（读取到内存）
let data = archiver::preview("archive.zip", "config.json", None)?;

// 创建带密码的压缩包
let opts = CreateOptions {
    password: Some("secret".into()),
    ..Default::default()
};
archiver::create("secure.zip", &[PathBuf::from("private/")], Some(&opts))?;

// 添加文件到已有压缩包
archiver::add("backup.tar.gz", &[PathBuf::from("new_file.txt")], None)?;

// 分包和合并
let parts = archiver::split_archive("huge.zip", 10_000_000, "./parts")?;
archiver::merge_parts(&parts, "restored.zip")?;
```

### 核心依赖

| 依赖 | 用途 |
|------|------|
| `zip` v2 | ZIP 读写，支持 AES 加密 |
| `tar` v0.4 | TAR 归档处理 |
| `flate2` / `bzip2` / `xz2` / `zstd` | 各类压缩算法 |
| `sevenz-rust` v0.6 | 7-Zip 读写及密码支持 |
| `unrar` v0.5 | RAR 读取（内置 C 库） |
