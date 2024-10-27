use thiserror::Error;

#[derive(Error, Debug)]
pub enum FixError {
    #[error("Invalid message format")]
    InvalidFormat,
    #[error("Invalid checksum")]
    InvalidChecksum,
    #[error("Missing required field: {0}")]
    MissingField(u32),
    #[error("Invalid field value")]
    InvalidFieldValue,
    #[error("Invalid body length")]
    InvalidBodyLength,
}