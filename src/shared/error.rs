use log4rs::config::runtime::{ConfigError, ConfigErrors};
use log::SetLoggerError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    ConfigurationFileError(#[from] twelf::Error),
    #[error(transparent)]
    LoggingConfigurationError(#[from] ConfigErrors),
    #[error(transparent)]
    LoggingInstantiationError(#[from] SetLoggerError),
    #[error("Invalid value '{offending_value}' provided for parameter '{parameter}': {message}")]
    ArgumentError {
        parameter : String,
        offending_value: String,
        message : String,
    },
    #[error("Invalid value '{as_number}' provided for AS number")]
    InvalidAsNumberError {
        as_number : i64,
    },
    #[error(transparent)]
    ParseIpAddressError(#[from] std::net::AddrParseError),
    #[error("Invalid value '{ip_address}' provided for IP number")]
    InvalidIpAddressError {
        ip_address : String,
    },
    #[error(transparent)]
    UnspecifiedError(#[from] anyhow::Error),
}
