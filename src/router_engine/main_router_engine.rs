use crate::router_engine::local_adress_matcher::{
    HostInterfacesLocalAddressMatcher, LocalAddressMatcher,
};
use crate::shared::prelude::Result;
use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};
use crate::shared::router_engine::RouterEngine;
use async_trait::async_trait;
use log::{info, warn};
use uuid::Uuid;

pub struct MainRouterEngine {
    local_address_matcher: Box<dyn LocalAddressMatcher>,
}

#[async_trait(?Send)]
impl RouterEngine for MainRouterEngine {
    async fn start(&self) -> crate::shared::prelude::Result<()> {
        Ok(())
    }

    async fn stop(&self) -> crate::shared::prelude::Result<()> {
        todo!()
    }

    async fn initial_configuration(
        &self,
        router_configuration: RouterConfiguration,
    ) -> crate::shared::prelude::Result<()> {
        for peer_configuration in router_configuration.peer_configurations {
            if Self::verify_local_addres_rule(&peer_configuration, &self.local_address_matcher)
                .await
            {
                info!("Peer {} passed local address rule", &peer_configuration);
            } else {
                warn!(
                    "Peer {} does not pass local address rule",
                    &peer_configuration
                );
            }
        }

        Ok(())
    }

    async fn add_peer(
        &self,
        peer_configuration: PeerConfiguration,
    ) -> crate::shared::prelude::Result<()> {
        if Self::verify_local_addres_rule(&peer_configuration, &self.local_address_matcher).await {
            info!("Peer {} passed local address rule", &peer_configuration);
        } else {
            warn!(
                "Peer {} does not pass local address rule",
                &peer_configuration
            );
        }

        Ok(())
    }

    async fn remove_peer(&self, peer_id: &Uuid) -> crate::shared::prelude::Result<()> {
        Ok(())
    }

    async fn await_termination(&self) -> () {
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

    async fn verify_local_addres_rule(
        peer_confguration: &PeerConfiguration,
        local_address_matcher: &Box<dyn LocalAddressMatcher>,
    ) -> bool {
        local_address_matcher
            .is_local_address(&peer_confguration.local_address.ip())
            .await
            && !local_address_matcher
                .is_local_address(&peer_confguration.remote_address.ip())
                .await
    }
}
