// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::config_file::config_file_definition::EngineConfigFile;
use crate::shared::config::config_provider::ConfigProvider;
use crate::shared::config::router_configuration::RouterConfiguration;
use crate::shared::error::Error::ConfigurationFileError;
use crate::shared::prelude::Result;
use log::info;

pub struct ConfigFileProvider {
    file_path: String,
}

impl ConfigProvider for ConfigFileProvider {
    fn provide_configuration(&self) -> Result<RouterConfiguration> {
        info!(
            "Configuration file provider starting with log file {}",
            &self.file_path
        );

        match EngineConfigFile::parse(&self.file_path) {
            Ok(engine_config_file) => {
                info!("Configuration file parsed successfully");

                TryInto::<RouterConfiguration>::try_into(engine_config_file)
            }
            Err(e) => Err(ConfigurationFileError(e)),
        }
    }
}

impl ConfigFileProvider {
    pub fn new(file_path: &str) -> Result<Box<dyn ConfigProvider>> {
        Ok(Box::new(ConfigFileProvider {
            file_path: file_path.to_string(),
        }))
    }
}
