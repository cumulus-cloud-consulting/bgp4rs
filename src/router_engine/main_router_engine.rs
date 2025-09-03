use log::{info, warn};
use crate::router_engine::local_adress_matcher::{
    HostInterfacesLocalAddressMatcher, LocalAddressMatcher,
};
use crate::shared::prelude::Result;
use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};
use crate::shared::router_engine::RouterEngine;
use uuid::Uuid;

pub struct MainRouterEngine {
    local_address_matcher: Box<dyn LocalAddressMatcher>,
}

impl RouterEngine for MainRouterEngine {
    fn start(&self) -> crate::shared::prelude::Result<()>
    where
        Self: Sized,
    {
        Ok(())
    }

    fn stop(&self) -> crate::shared::prelude::Result<()> {
        todo!()
    }

    fn initial_configuration(
        &self,
        router_configuration: RouterConfiguration,
    ) -> crate::shared::prelude::Result<()> {
        for peer_configuration in router_configuration.peer_configurations {
            if Self::verify_local_addres_rule(&peer_configuration, &self.local_address_matcher) {
                info!("Peer {} passed local address rule", &peer_configuration);
            } else {
                warn!("Peer {} does not pass local address rule", &peer_configuration);
            }
        }

        Ok(())
    }

    fn add_peer(&self, peer_configuration: PeerConfiguration) -> crate::shared::prelude::Result<()> {
        if Self::verify_local_addres_rule(&peer_configuration, &self.local_address_matcher) {
            info!("Peer {} passed local address rule", &peer_configuration);
        } else {
            warn!("Peer {} does not pass local address rule", &peer_configuration);
        }

        Ok(())
    }

    fn remove_peer(&self, peer_id: &Uuid) -> crate::shared::prelude::Result<()> {
        Ok(())
    }

    fn await_termination(&self) -> () {
        ()
    }
}

impl MainRouterEngine {
    pub fn new() -> Result<Box<dyn RouterEngine>> {
        match HostInterfacesLocalAddressMatcher::new() {
            Ok(local_address_matcher) => Ok(Box::new(MainRouterEngine {
                local_address_matcher,
            })),
            Err(error) => Err(error),
        }
    }

    fn verify_local_addres_rule(peer_confguration: &PeerConfiguration,
                                local_address_matcher : &Box<dyn LocalAddressMatcher>) -> bool {
        local_address_matcher.is_local_address(&peer_confguration.local_address.ip())
            && !local_address_matcher.is_local_address(&peer_confguration.remote_address.ip())
    }
}
