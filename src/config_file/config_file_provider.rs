use std::rc::Rc;
use log::info;
use crate::shared::config_provider::ConfigProvider;
use crate::shared::prelude::Result;
use crate::shared::router_configuration::RouterConfiguration;
use crate::shared::router_engine::RouterEngine;
use crate::config_file::config_file_definition::EngineConfigFile;
use crate::shared::error::Error::ConfigurationFileError;

pub struct ConfigFileProvider {
    router_engine: Rc<Box<dyn RouterEngine>>,
    file_path: String,
}

impl ConfigProvider for ConfigFileProvider {
    fn provide_configuration(&self) -> Result<RouterConfiguration> {
        info!("Configuration file provider starting with log file {}", &self.file_path);

        match EngineConfigFile::parse(&self.file_path) {
            Ok(engine_config_file) => {
                info!("Configuration file parsed successfully");
                
                TryInto::<RouterConfiguration>::try_into(engine_config_file)
            },
            Err(e) => Err(ConfigurationFileError(e)),
        }
    }
}

impl ConfigFileProvider {
    pub fn new(router_engine: &Rc<Box<dyn RouterEngine>>, file_path: &str) -> Result<Box<dyn ConfigProvider>> {
        Ok(Box::new(ConfigFileProvider {
            router_engine: Rc::clone(router_engine),
            file_path: file_path.to_string(),
        }))
    }
}

