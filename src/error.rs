use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("File not found or could not be read: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse system model JSON: {0}")]
    DeserializationError(#[from] serde_json::Error),

    #[error("Failed to build internal domain model: {0}")]
    ModelConstructionError(String),

    #[error("Failed to build VRM system model:")]
    VrmSystemModelConstructionError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConversionError {
    UnknownSchedulerType(String),
    UnknownRmsType(String),
    VrmConstructionError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
