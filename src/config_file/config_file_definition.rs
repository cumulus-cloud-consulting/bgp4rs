// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::bgp::as_number::AsNumber;
use crate::shared::config::router_configuration::{PeerConfiguration, RouterConfiguration};
use crate::shared::error::Error::InvalidIpAddressError;
use crate::shared::network::socket_addr_spec::SocketAddressSpec;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use twelf::{config, Error, Layer};
use uuid::Uuid;

#[config]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfigFile {
    local_as: AsNumber,
    peers: Vec<PeerConfigFile>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PeerConfigFile {
    peer_as: AsNumber,
    peer_name: String,
    peer_address: SocketAddressSpec,
    local_address: SocketAddressSpec,
}

impl EngineConfigFile {
    pub fn parse(file_path: &str) -> Result<EngineConfigFile, Error> {
        EngineConfigFile::with_layers(&[
            Layer::Yaml(file_path.into()),
            Layer::Env(Some("BGP4RS_".to_string())),
        ])
    }
}

impl Display for PeerConfigFile {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Peer name {}, Peer AS {}, Local address {}, peer address {}",
            self.peer_name, self.peer_as, self.local_address, self.peer_address
        )
    }
}

const BGP4_DEFAULT_PORT_NUMBER: u16 = 179;

impl TryInto<PeerConfiguration> for PeerConfigFile {
    type Error = crate::shared::prelude::Error;

    fn try_into(self) -> Result<PeerConfiguration, Self::Error> {
        info!("Processing peer configuration file entry {}", &self);

        match TryInto::<SocketAddr>::try_into(
            self.local_address
                .with_default_port(BGP4_DEFAULT_PORT_NUMBER),
        ) {
            Ok(local_address) => match TryInto::<SocketAddr>::try_into(
                self.peer_address
                    .with_default_port(BGP4_DEFAULT_PORT_NUMBER),
            ) {
                Ok(remote_address) => {
                    let local_ip_address = &local_address.ip();
                    let remote_ip_address = &remote_address.ip();

                    if local_ip_address == remote_ip_address {
                        error!("Local and peer IP pointing to same host");

                        return Err(InvalidIpAddressError {
                            ip_address: local_address.to_string(),
                        });
                    }

                    let peer_configuration = PeerConfiguration {
                        peer_id: Uuid::new_v4(),
                        peer_name: self.peer_name.clone(),
                        remote_as_number: self.peer_as,
                        remote_address,
                        local_address,
                    };

                    info!("Peer configuration {}", &peer_configuration);

                    Ok(peer_configuration)
                }
                Err(err) => {
                    error!("Failed to parse peer address: {}", err);

                    Err(err)
                }
            },
            Err(err) => {
                error!("Failed to parse local address: {}", err);

                Err(err)
            }
        }
    }
}

impl TryInto<RouterConfiguration> for EngineConfigFile {
    type Error = crate::shared::prelude::Error;

    fn try_into(self) -> Result<RouterConfiguration, Self::Error> {
        let mut peers: Vec<PeerConfiguration> = Vec::new();

        for peer_config in self.peers {
            match peer_config.try_into() {
                Ok(peer) => peers.push(peer),
                Err(err) => return Err(err),
            }
        }

        Ok(RouterConfiguration {
            local_as_number: self.local_as,
            peer_configurations: peers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn should_convert_peer_config_file() {
        let peer_spec = PeerConfigFile {
            peer_as: AsNumber::Small(1234),
            peer_name: "Some peer".to_string(),
            peer_address: SocketAddressSpec::new(&"192.168.1.1", &None),
            local_address: SocketAddressSpec::new(&"192.168.2.2", &None),
        };

        match TryInto::<PeerConfiguration>::try_into(peer_spec) {
            Ok(peer_config) => {
                assert_eq!(peer_config.remote_as_number, AsNumber::Small(1234));
                assert_eq!(peer_config.peer_name, "Some peer".to_string());
                assert_eq!(
                    peer_config.remote_address,
                    SocketAddr::new(Ipv4Addr::new(192, 168, 1, 1).into(), 179)
                );
                assert_eq!(
                    peer_config.local_address,
                    SocketAddr::new(Ipv4Addr::new(192, 168, 2, 2).into(), 179)
                );
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_peer_config_file_with_local_address_is_loopback() {
        let peer_spec = PeerConfigFile {
            peer_as: AsNumber::Small(1234),
            peer_name: "Some peer".to_string(),
            peer_address: SocketAddressSpec::new(&"192.168.1.1", &None),
            local_address: SocketAddressSpec::new(&"127.0.0.1", &None),
        };

        match TryInto::<PeerConfiguration>::try_into(peer_spec) {
            Ok(peer_config) => panic!("Should not convert to peer config {peer_config}"),
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_peer_config_file_with_remote_address_is_loopback() {
        let peer_spec = PeerConfigFile {
            peer_as: AsNumber::Small(1234),
            peer_name: "Some peer".to_string(),
            peer_address: SocketAddressSpec::new(&"127.0.0.1", &None),
            local_address: SocketAddressSpec::new(&"192.168.2.2", &None),
        };

        match TryInto::<PeerConfiguration>::try_into(peer_spec) {
            Ok(peer_config) => panic!("Should not convert to peer config {peer_config}"),
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_peer_config_file_with_equal_local_and_remote_address() {
        let peer_spec = PeerConfigFile {
            peer_as: AsNumber::Small(1234),
            peer_name: "Some peer".to_string(),
            peer_address: SocketAddressSpec::new(&"192.168.1.1", &None),
            local_address: SocketAddressSpec::new(&"192.168.1.1", &None),
        };

        match TryInto::<PeerConfiguration>::try_into(peer_spec) {
            Ok(peer_config) => panic!("Should not convert to peer config {peer_config}"),
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_router_config_file() {
        let router_spec = EngineConfigFile {
            local_as: AsNumber::Small(2345),
            peers: vec![PeerConfigFile {
                peer_as: AsNumber::Small(1234),
                peer_name: "Some peer".to_string(),
                peer_address: SocketAddressSpec::new(&"192.168.1.1", &None),
                local_address: SocketAddressSpec::new(&"192.168.2.2", &None),
            }],
        };

        match TryInto::<RouterConfiguration>::try_into(router_spec) {
            Ok(mut router_config) => {
                assert_eq!(router_config.local_as_number, AsNumber::Small(2345));
                assert_eq!(router_config.peer_configurations.len(), 1);

                let peer_config = &router_config.peer_configurations[0];

                assert_eq!(peer_config.remote_as_number, AsNumber::Small(1234));
                assert_eq!(peer_config.peer_name, "Some peer".to_string());
                assert_eq!(
                    peer_config.remote_address,
                    SocketAddr::new(Ipv4Addr::new(192, 168, 1, 1).into(), 179)
                );
                assert_eq!(
                    peer_config.local_address,
                    SocketAddr::new(Ipv4Addr::new(192, 168, 2, 2).into(), 179)
                );
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_router_config_file_with_equal_local_and_remote_address() {
        let router_spec = EngineConfigFile {
            local_as: AsNumber::Small(2345),
            peers: vec![PeerConfigFile {
                peer_as: AsNumber::Small(1234),
                peer_name: "Some peer".to_string(),
                peer_address: SocketAddressSpec::new(&"192.168.1.1", &None),
                local_address: SocketAddressSpec::new(&"192.168.1.1", &None),
            }],
        };

        match TryInto::<RouterConfiguration>::try_into(router_spec) {
            Ok(mut router_config) => panic!("Should not convert to router configuration"),
            Err(err) => {}
        }
    }
}
