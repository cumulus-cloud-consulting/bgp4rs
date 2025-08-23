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
}
