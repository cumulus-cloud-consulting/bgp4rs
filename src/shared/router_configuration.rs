// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::as_number::AsNumber;
use core::net::SocketAddr;
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq)]
pub struct RouterConfiguration {
    pub local_as_number: AsNumber,
    pub peer_configurations: Vec<PeerConfiguration>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct PeerConfiguration {
    pub peer_id: Uuid,
    pub peer_name: String,
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
    pub remote_as_number: AsNumber,
}

impl Display for PeerConfiguration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Peer name: {}, local address {}, remote address {}, remote peer AS {}",
            self.peer_name, self.local_address, self.remote_address, self.remote_as_number
        )
    }
}
