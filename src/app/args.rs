use std::path::Path;
use crate::config_file::config_file_provider::ConfigFileProvider;
use crate::shared::config_provider::ConfigProvider;
use crate::shared::error::Error::ArgumentError;
use crate::shared::prelude::Result;
use crate::shared::router_engine::RouterEngine;
use clap::{Parser, ValueEnum};
use std::rc::Rc;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, default_value_t = String::from("./config/app.yaml"), env("APP_CONFIG_PATH"))]
    pub file_path: String,

    /// configuration type to be applied. Defaults to file based if not specified
    #[arg(value_enum, short, long, default_value_t = ConfigType::File)]
    pub config_type: ConfigType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConfigType {
    File,
}

pub fn parse() -> Result<Args> {
    let args = Args::parse();

    match args.config_type {
        ConfigType::File => {
            let config_file_path = Path::new(&args.file_path.as_str()).to_path_buf();

            if !config_file_path.exists() {
                Err(ArgumentError {
                    parameter: "-f".to_string(),
                    offending_value: args.file_path.clone(),
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
        router_engine: &Rc<Box<dyn RouterEngine>>,
    ) -> Result<Box<dyn ConfigProvider>> {
        match self.config_type {
            ConfigType::File => ConfigFileProvider::new(router_engine, self.file_path.as_str()),
        }
    }
}
