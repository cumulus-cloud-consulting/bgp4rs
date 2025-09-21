// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
//! This module contains the actual command-line arguments handling code.
//!
//!  The list of supported command-line arguments is documented in the [`crate::main`]
//!
use crate::config_file::config_file_provider::ConfigFileProvider;
use crate::shared::config::config_provider::ConfigProvider;
use crate::shared::error::Error::{
    ArgumentError, LoggingConfigurationError, LoggingInstantiationError, UnspecifiedError,
};
use crate::shared::network::socket_addr_spec::SocketAddressSpec;
use crate::shared::prelude::Result;
use crate::shared::services::router_engine::RouterEngine;
use clap::{Parser, ValueEnum};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::path::Path;
use std::sync::Arc;

/// Main structure holding all supported command-line options and reasonable defaults
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, default_value_t = String::from("./config/app.yaml"), env("ROUTER_CONFIG_PATH"))]
    pub router_config_path: String,

    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, env("LOG_CONFIG_PATH"))]
    pub log_config_path: Option<String>,

    /// configuration type to be applied. Defaults to file based if not specified
    #[arg(value_enum, short, long, default_value_t = ConfigType::File)]
    pub config_type: ConfigType,

    /// Bind address for embedded management web server
    #[arg(long, env("MANAGEMENT_SERVER_BIND_ADDR"))]
    pub management_server_bind_addr: Option<String>,

    /// Bind address for embedded management web server
    #[arg(long, env("MANAGEMENT_SERVER_PORT"))]
    pub management_server_port: Option<u16>,

    /// Bind address for embedded API web server
    #[arg(long, env("API_SERVER_BIND_ADDR"))]
    pub api_server_bind_addr: Option<String>,

    /// Bind address for embedded API web server
    #[arg(long, env("API_SERVER_PORT"))]
    pub api_server_port: Option<u16>,
}

/// Supported types of peer configuration sources
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConfigType {
    /// Select a file based peer configuration source
    File,
}

/// Parse the passed command-line and execute additional validations based on the selected peer
/// configuration source.
///
/// The supported validations are:
/// - In case of a [`ConfigType::File`] source, validate that the passed path denotes an existing
/// file
///
pub fn parse() -> Result<Args> {
    let args = Args::parse();

    match args.config_type {
        ConfigType::File => {
            let config_file_path = Path::new(&args.router_config_path.as_str()).to_path_buf();

            if !config_file_path.exists() {
                Err(ArgumentError {
                    parameter: "-f".to_string(),
                    offending_value: args.router_config_path.clone(),
                    message: "Config file does not exist.".to_string(),
                })
            } else {
                Ok(args)
            }
        }
    }
}

impl Args {
    /// Obtain the [`crate::shared::config::config_provider::ConfigProvider`] instance which
    /// handles peer configuration according to the selected [`ConfigType`] option
    pub fn config_provider(
        &self,
        _router_engine: &Arc<Box<dyn RouterEngine + Send + Sync>>,
    ) -> Result<Box<dyn ConfigProvider>> {
        match self.config_type {
            ConfigType::File => ConfigFileProvider::new(self.router_config_path.as_str()),
        }
    }

    /// Initialize the logging subsystem taking the logging configuration file into account
    /// in case a file path to a logging configuration file is provided.
    ///
    /// The algorithm to initialize the logging subsystem is as follows:
    /// - if a path to a logging configuration file is specified and the path points to
    /// an existing file, it is attempted to initialize the logging subsystem using this file.
    /// If this fails, escalate the error to the caller.
    /// - if a path to a logging configuration file is specified and the path points to
    /// a non-existing file, then initialize the logging subsystem with a reasonable
    /// default configuration
    /// - if a path to a logging configuration file is not specified, then initialize the
    /// logging subsystem with a reasonable default configuration.
    ///
    pub fn initialize_logging(&self) -> Result<()> {
        match self.log_config_path {
            Some(ref log_config_path) => {
                let log_file_path = Path::new(log_config_path.as_str()).to_path_buf();

                if !log_file_path.exists() {
                    Self::initialize_logging_with_defaults()
                } else {
                    match log4rs::init_file(log_file_path, Default::default()) {
                        Ok(()) => Ok(()),
                        Err(err) => Err(UnspecifiedError(err)),
                    }
                }
            }
            None => Self::initialize_logging_with_defaults(),
        }
    }

    /// Initialize the logging subsystem with a resonable default configuration:
    /// - Log any messages to stdout
    /// - Set the minimal logging level to [`LevelFilter::Info`]
    fn initialize_logging_with_defaults() -> Result<()> {
        let stdout = ConsoleAppender::builder().build();

        match Config::builder()
            .appender(Appender::builder().build("stdout", Box::new(stdout)))
            .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        {
            Ok(config) => match log4rs::init_config(config) {
                Ok(_) => Ok(()),
                Err(err) => Err(LoggingInstantiationError(err)),
            },
            Err(e) => Err(LoggingConfigurationError(e)),
        }
    }

    /// Assemble the values of the given command-line arguments
    /// [`Args::management_server_bind_addr`] and [`Args::management_server_port`]
    /// into a [`SocketAddressSpec`] if both values are provided
    pub fn management_api_server_binding(&self) -> Option<SocketAddressSpec> {
        if let Some(ip_addr_spec) = self.management_server_bind_addr.as_ref()
            && let Some(port_number) = self.management_server_port
        {
            Some(SocketAddressSpec::new_allowing_localhost(
                ip_addr_spec,
                &Some(port_number),
            ))
        } else {
            None
        }
    }

    /// Assemble the values of the given command-line arguments
    /// [`Args::api_server_bind_addr`] and [`Args::api_server_port`]
    /// into a [`SocketAddressSpec`] if both values are provided
    pub fn public_api_server_binding(&self) -> Option<SocketAddressSpec> {
        if let Some(ip_addr_spec) = self.api_server_bind_addr.as_ref()
            && let Some(port_number) = self.api_server_port
        {
            Some(SocketAddressSpec::new_allowing_localhost(
                ip_addr_spec,
                &Some(port_number),
            ))
        } else {
            None
        }
    }
}
