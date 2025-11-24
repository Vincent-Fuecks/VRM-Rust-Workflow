use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File not found or could not be read: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse system model JSON: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("Failed to build internal domain model: {0}")]
    ModelConstructionError(String),

    #[error("Failed to build VRM system model: {0}")]
    VrmSystemModelConstructionError,
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::VrmSystemModelConstructionError(
            "An unspecified operation failed during VRM system model construction.".to_string(),
        )
    }
}

pub type Result<T> = std::result::Result<T, Error>;
