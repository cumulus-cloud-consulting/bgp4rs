use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};
use crate::shared::router_engine::RouterEngine;
use uuid::Uuid;

pub struct MainRouterEngine {
    
}

impl RouterEngine for MainRouterEngine {
    fn start(&self) -> crate::shared::prelude::Result<()>
    where
        Self: Sized
    {
        todo!()
    }

    fn stop(&self) -> crate::shared::prelude::Result<()> {
        todo!()
    }

    fn initial_configuration(&self, router_configuration: &RouterConfiguration) -> crate::shared::prelude::Result<()> {
        todo!()
    }

    fn add_peer(&self, peer: &PeerConfiguration) -> crate::shared::prelude::Result<()> {
        todo!()
    }

    fn remove_peer(&self, peer_id: &Uuid) -> crate::shared::prelude::Result<()> {
        todo!()
    }
}

impl MainRouterEngine {
    pub fn new() -> Box<dyn RouterEngine> {
        Box::new(MainRouterEngine {})
    }
}