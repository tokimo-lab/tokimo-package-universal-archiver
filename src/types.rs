use std::fmt;

/// Information about an entry in an archive
#[derive(Debug, Clone)]
pub struct EntryInfo {
    pub path: String,
    pub size: u64,
    pub compressed_size: Option<u64>,
    pub is_dir: bool,
    pub modified: Option<String>,
    pub encrypted: bool,
}

/// Supported archive formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    Zip,
    Tar,
    TarGz,
    TarBz2,
    TarXz,
    TarZst,
    SevenZ,
    Rar,
    Gz,
    Bz2,
    Xz,
    Zst,
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Zip => ".zip",
            Self::Tar => ".tar",
            Self::TarGz => ".tar.gz",
            Self::TarBz2 => ".tar.bz2",
            Self::TarXz => ".tar.xz",
            Self::TarZst => ".tar.zst",
            Self::SevenZ => ".7z",
            Self::Rar => ".rar",
            Self::Gz => ".gz",
            Self::Bz2 => ".bz2",
            Self::Xz => ".xz",
            Self::Zst => ".zst",
        }
    }

    pub fn is_single_file(&self) -> bool {
        matches!(self, Self::Gz | Self::Bz2 | Self::Xz | Self::Zst)
    }

    pub fn supports_password(&self) -> bool {
        matches!(self, Self::Zip | Self::SevenZ | Self::Rar)
    }

    pub fn supports_write(&self) -> bool {
        !matches!(self, Self::Rar)
    }
}

impl fmt::Display for ArchiveFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zip => write!(f, "ZIP"),
            Self::Tar => write!(f, "TAR"),
            Self::TarGz => write!(f, "TAR.GZ"),
            Self::TarBz2 => write!(f, "TAR.BZ2"),
            Self::TarXz => write!(f, "TAR.XZ"),
            Self::TarZst => write!(f, "TAR.ZST"),
            Self::SevenZ => write!(f, "7Z"),
            Self::Rar => write!(f, "RAR"),
            Self::Gz => write!(f, "GZ"),
            Self::Bz2 => write!(f, "BZ2"),
            Self::Xz => write!(f, "XZ"),
            Self::Zst => write!(f, "ZST"),
        }
    }
}

/// Options for opening/reading an archive
#[derive(Debug, Clone, Default)]
pub struct OpenOptions {
    pub password: Option<String>,
}

/// Options for creating/writing an archive
#[derive(Debug, Clone, Default)]
pub struct CreateOptions {
    pub password: Option<String>,
    pub compression_level: Option<i32>,
}
