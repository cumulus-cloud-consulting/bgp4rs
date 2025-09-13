// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use log::SetLoggerError;
use log4rs::config::runtime::ConfigErrors;
use std::net::SocketAddr;
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
        parameter: String,
        offending_value: String,
        message: String,
    },
    #[error("Invalid value '{as_number}' provided for AS number")]
    InvalidAsNumberError { as_number: i64 },
    #[error(transparent)]
    ParseIpAddressError(#[from] std::net::AddrParseError),
    #[error("Invalid value '{ip_address}' provided for IP number")]
    InvalidIpAddressError { ip_address: String },
    #[error("Cannot bind to '{bind_addr}' provided for IP number")]
    InvalidBindAddressError { bind_addr: SocketAddr },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    UnspecifiedError(#[from] anyhow::Error),
}
