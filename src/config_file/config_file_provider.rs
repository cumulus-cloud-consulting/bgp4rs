use std::rc::Rc;

use crate::shared::config_provider::ConfigProvider;
use crate::shared::prelude::Result;
use crate::shared::router_configuration::RouterConfiguration;
use crate::shared::router_engine::RouterEngine;
use crate::config_file::config_file_definition::EngineConfigFile;
use crate::shared::error::Error::ConfigError;

pub struct ConfigFileProvider {
    router_engine: Rc<Box<dyn RouterEngine>>,
    file_path: String,
}

impl ConfigProvider for ConfigFileProvider {
    fn provide_configuration(&self) -> Result<RouterConfiguration> {
        match EngineConfigFile::parse(&self.file_path) {
            Ok(engine_config_file) => {
                match EngineConfigFile::parse(&self.file_path) {
                    Ok(engine_config) => TryInto::<RouterConfiguration>::try_into(engine_config_file),
                    Err(e) => Err(ConfigError(e)),
                }
            },
            Err(error) => Err(ConfigError(error)),
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

