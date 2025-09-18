use crate::router_engine::bgp_packet::BgpPacket;
use std::net::SocketAddr;

// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
pub struct PeerConnectionManager {}

pub struct PeerIdentification {
    local_address: SocketAddr,
    remote_address: SocketAddr,
}

pub struct BgpPacketTransmission {
    peer_identification: PeerIdentification,
    bgp_packet: BgpPacket,
}

/// Supported verbs send to the peer connection manager. The verbs are either send from
/// the main router engine or from the peer BGP FSM
pub enum PeerConnectionVerb {
    /// Peer is added to the list of connectable peers. This may spawn an additional
    /// TCP listener endpoint if no previously existing listener for that local address.
    AddPeer(PeerIdentification),
    /// Peer is removed from the list of connectable peers. This may cause the TCP listener on the
    /// local address to be withdrawn if the last remaining peer is removed through this message
    RemovePeer(PeerIdentification),
    /// Send a BGP protocol packet to the remote peer. It is assumed that there is at leat one
    /// active TCP connection to the peer. In case of a connection collision, the locally originated
    /// connection is given priority for sending out the message.
    SendPacket(BgpPacketTransmission),
    /// Initiate a connection to the peer from the local address. If there is already a locally
    /// originated connection available or a connection attempt is under way, this message
    /// is ignored.
    AttemptConnection(PeerIdentification),
    /// Close all connections to the remote peer. This should terminate only one connection in
    /// normal operation mode but may close both connections in case of the connection collision.
    CloseConnection(PeerIdentification),
    /// Close the connection to the peer which originated from the local end. This message is
    /// usually sent as part of the connection collision resolution process.
    CloseOriginatedConnection(PeerIdentification),
    /// Close the connection to the peer which originated from the remote peer. This message is
    /// usually sent as part of the connection collision resolution process.
    CloseReceivedConnection(PeerIdentification),
    /// Completely shutdown the connection manager by closing all established TCP connections
    /// and by withdrawing all local TCP listener endpoints.
    Shutdown,
}
