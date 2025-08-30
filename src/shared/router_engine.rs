use uuid::Uuid;
use crate::shared::prelude::Result;
use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};

/// Trait definition for the router engine to implement
///
pub trait RouterEngine {
    /// Start the router engine:
    ///
    fn start(&self) -> Result<()>;

    /// Stop the router engine
    fn stop(&self) -> Result<()>;

    /// Provide initial configuration to the router engine
    fn initial_configuration(&self, router_configuration : &RouterConfiguration) -> Result<()>;

    /// Add a peer to a running engine
    fn add_peer(&self, peer : &PeerConfiguration) -> Result<()>;

    /// Remove a peer from a running engine
    fn remove_peer(&self, peer_id : &Uuid) -> Result<()>;
}