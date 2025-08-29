use crate::shared::as_number::AsNumber;
use core::net::SocketAddr;
use uuid_rs::UUID;

#[derive(Debug, Eq, PartialEq)]
pub struct RouterConfiguration {
    pub local_as_number: AsNumber,
    pub peer_configurations: Vec<PeerConfiguration>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PeerConfiguration {
    pub peer_id : UUID,
    pub local_address: SocketAddr,
    pub remote_address : SocketAddr,
    pub remote_as_number : AsNumber,
}