use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("Password required for this archive")]
    PasswordRequired,

    #[error("Invalid password")]
    InvalidPassword,

    #[error("Entry not found: {0}")]
    EntryNotFound(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("Feature not supported for this format: {0}")]
    NotSupported(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, ArchiveError>;
