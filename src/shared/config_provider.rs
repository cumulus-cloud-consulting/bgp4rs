use crate::shared::prelude::Result;
use crate::shared::router_configuration::RouterConfiguration;

pub trait ConfigProvider {
    fn provide_configuration(&self) -> Result<RouterConfiguration>;
}