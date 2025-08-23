use clap::{Parser, ValueEnum};
use crate::shared::prelude::Result;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to config file. Required if file-based configuration is used
    #[arg(short, long, default_value_t = String::from("./config/app.yaml"), env("APP_CONFIG_PATH"))]
    pub file_path: String,

    /// configuration type to be applied. Defaults to file based if not specified
    #[arg(value_enum, short, long, default_value_t = ConfigType::File)]
    pub config_type : ConfigType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConfigType {
    File,
}

pub fn parse() -> Result<Args> {
    let args = Args::parse();

    match args.config_type {
        ConfigType::File => {

        }
    }

    Ok(args)
}