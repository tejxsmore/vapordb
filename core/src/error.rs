use thiserror::Error;

#[derive(Error, Debug)]
pub enum VaporDBError {
    #[error("Key not found")]
    KeyNotFound,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error), // Automatically converts std::io::Error into VaporDBError::Io

    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Type mismatch error: {0}")]
    TypeMismatch(String),

    #[error("Compaction error: {0}")]
    CompactionFailed(String),
}

pub type Result<T> = std::result::Result<T, VaporDBError>;
