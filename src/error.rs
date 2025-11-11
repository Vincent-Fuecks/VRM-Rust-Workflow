use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File not found or could not be read: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse system model JSON: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("Failed to build internal domain model: {0}")]
    ModelConstructionError(String),
}

/// A convenience type alias for `Result` with our library's `Error` type.
pub type Result<T> = std::result::Result<T, Error>;