use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ConfigError(#[from] twelf::Error),
    #[error("Invalid value '{offending_value}' provided for parameter '{parameter}': {message}")]
    ArgumentError {
        parameter : String,
        offending_value: String,
        message : String,
    },
    #[error("Invalid value '{as_number}' provided for AS number")]
    InvalidAsNumerError {
        as_number : i64,
    },
}
