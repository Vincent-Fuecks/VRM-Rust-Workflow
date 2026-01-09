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

    #[error("Conversion error: {0}")]
    Conversion(#[from] ConversionError),
}

#[derive(Debug, Error)]
pub enum ConversionError {
    #[error("Unknown scheduler type: {0}")]
    UnknownSchedulerType(String),

    #[error("Unknown RMS type: {0}")]
    UnknownRmsType(String),

    #[error("VRM construction error: {0}")]
    VrmConstructionError(String),

    #[error("ADC construction error: {0}")]
    AdcConstructionError(String),

    #[error("System construction error: {0}")]
    SystemConstructionError(String),

    #[error("A system error occurred during conversion: {0}")]
    SystemError(#[from] Box<Error>),
}

pub type Result<T> = std::result::Result<T, Error>;
