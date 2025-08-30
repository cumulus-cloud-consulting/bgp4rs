use crate::shared::as_number::AsNumber;
use core::net::SocketAddr;
use std::fmt::{write, Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq)]
pub struct RouterConfiguration {
    pub local_as_number: AsNumber,
    pub peer_configurations: Vec<PeerConfiguration>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PeerConfiguration {
    pub peer_id : Uuid,
    pub peer_name : String,
    pub local_address: SocketAddr,
    pub remote_address : SocketAddr,
    pub remote_as_number : AsNumber,
}

impl Display for PeerConfiguration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Peer name: {}, local address {}, remote address {}, remote peer AS {}",
                 self.peer_name,self.local_address,self.remote_address,self.remote_as_number)

    }
}