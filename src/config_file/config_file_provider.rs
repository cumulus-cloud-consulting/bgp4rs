use std::rc::Rc;

use crate::shared::config_provider::ConfigProvider;
use crate::shared::prelude::Result;
use crate::shared::router_configuration::RouterConfiguration;
use crate::shared::router_engine::RouterEngine;

pub struct ConfigFileProvider {
    router_engine: Rc<Box<dyn RouterEngine>>,
    file_path: String,
}

impl ConfigProvider for ConfigFileProvider {
    fn provide_configuration(&self) -> Result<RouterConfiguration> {
        todo!()
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

