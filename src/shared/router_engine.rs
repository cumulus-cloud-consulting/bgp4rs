use crate::shared::prelude::Result;
use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};
use async_trait::async_trait;
use uuid::Uuid;

/// Trait definition for the router engine to implement
///
///
#[async_trait(?Send)]
pub trait RouterEngine {
    /// Start the router engine:
    ///
    async fn start(&self) -> Result<()>;

    /// Stop the router engine
    async fn stop(&self) -> Result<()>;

    /// Provide initial configuration to the router engine
    async fn initial_configuration(&self, router_configuration: RouterConfiguration) -> Result<()>;

    /// Add a peer to a running engine
    async fn add_peer(&self, peer: PeerConfiguration) -> Result<()>;

    /// Remove a peer from a running engine
    async fn remove_peer(&self, peer_id: &Uuid) -> Result<()>;

    /// Await router engine termination
    async fn await_termination(&self) -> ();
}
