// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::config_file::config_file_provider::ConfigFileProvider;
use crate::shared::config_provider::ConfigProvider;
use crate::shared::error::Error::{
    ArgumentError, LoggingConfigurationError, LoggingInstantiationError, UnspecifiedError,
};
use crate::shared::prelude::Result;
use crate::shared::router_engine::RouterEngine;
use crate::shared::socket_addr_spec::SocketAddressSpec;
use clap::{Parser, ValueEnum};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use log4rs::Config;
use std::path::Path;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, default_value_t = String::from("./config/app.yaml"), env("ROUTER_CONFIG_PATH"))]
    pub router_config_path: String,

    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, default_value_t = String::from("./config/log4rs.yaml"), env("LOG_CONFIG_PATH"))]
    pub log_config_path: String,

    /// configuration type to be applied. Defaults to file based if not specified
    #[arg(value_enum, short, long, default_value_t = ConfigType::File)]
    pub config_type: ConfigType,

    /// Bind address for embedded web server
    #[arg(long, env("WEB_SERVER_BIND_ADDR"))]
    pub http_server_bind_addr: Option<String>,

    /// Bind address for embedded web server
    #[arg(long, env("WEB_SERVER_PORT"))]
    pub http_server_port: Option<u16>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConfigType {
    File,
}

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
    pub fn config_provider(
        &self,
        _router_engine: &Arc<Box<dyn RouterEngine + Send + Sync>>,
    ) -> Result<Box<dyn ConfigProvider>> {
        match self.config_type {
            ConfigType::File => ConfigFileProvider::new(self.router_config_path.as_str()),
        }
    }

    pub fn initialize_logging(&self) -> Result<()> {
        let log_file_path = Path::new(&self.log_config_path.as_str()).to_path_buf();

        if !log_file_path.exists() {
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
        } else {
            match log4rs::init_file(log_file_path, Default::default()) {
                Ok(()) => Ok(()),
                Err(err) => Err(UnspecifiedError(err)),
            }
        }
    }

    pub fn http_server_binding(&self) -> Option<SocketAddressSpec> {
        if let Some(ip_addr_spec) = self.http_server_bind_addr.as_ref()
            && let Some(port_number) = self.http_server_port
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
