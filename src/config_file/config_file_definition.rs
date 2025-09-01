use crate::shared::as_number::AsNumber;
use crate::shared::error::Error::{InvalidIpAddressError, ParseIpAddressError};
use crate::shared::router_configuration::{PeerConfiguration, RouterConfiguration};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, write};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use std::str::FromStr;
use std::string::ParseError;
use twelf::{Error, Layer, config};
use uuid::Uuid;

#[config]
#[derive(Default)]
#[serde(rename_all = "camelCase")]
pub struct EngineConfigFile {
    local_as: AsNumber,
    peers: Vec<PeerConfigFile>,
}

#[derive(Deserialize, Serialize, Default)]
pub struct PeerConfigFile {
    peer_as: AsNumber,
    peer_name: String,
    peer_address: SocketAddressSpec,
    local_address: SocketAddressSpec,
}

#[derive(Deserialize, Serialize, Default)]
pub struct SocketAddressSpec {
    ip_address: String,
    port_number: u16,
}

impl EngineConfigFile {
    pub fn parse(file_path: &str) -> Result<EngineConfigFile, Error> {
        EngineConfigFile::with_layers(&[
            Layer::Yaml(file_path.into()),
            Layer::Env(Some("BGP4RS_".to_string())),
        ])
    }
}

impl Display for SocketAddressSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IP Address {}, port number{}",
            &self.ip_address, &self.port_number
        )
    }
}

impl TryInto<SocketAddr> for SocketAddressSpec {
    type Error = crate::shared::prelude::Error;

    fn try_into(self) -> Result<SocketAddr, Self::Error> {
        if self.ip_address.is_empty() {
            return Err(InvalidIpAddressError {
                ip_address: self.ip_address,
            });
        }

        match IpAddr::from_str(self.ip_address.as_str()) {
            Ok(addr) => {
                if addr.is_loopback() || addr.is_multicast() {
                    return Err(InvalidIpAddressError {
                        ip_address: self.ip_address,
                    });
                }

                let port_number = if self.port_number == 0 {
                    179
                } else {
                    self.port_number
                };

                Ok(SocketAddr::new(addr, port_number))
            }
            Err(err) => Err(ParseIpAddressError(err)),
        }
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

impl TryInto<PeerConfiguration> for PeerConfigFile {
    type Error = crate::shared::prelude::Error;

    fn try_into(self) -> Result<PeerConfiguration, Self::Error> {
        info!("Processing peer configuration file entry {}", &self);

        match TryInto::<SocketAddr>::try_into(self.local_address) {
            Ok(local_address) => match TryInto::<SocketAddr>::try_into(self.peer_address) {
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
                },
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
    use std::net::{Ipv4Addr, SocketAddrV6};

    #[test]
    fn should_convert_ipv4_address_spec_without_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
                assert_eq!(addr.port(), 179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv4_address_spec_with_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: 1179,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
                assert_eq!(addr.port(), 1179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_ipv4_address_spec_for_localhost() {
        let spec = SocketAddressSpec {
            ip_address: "127.0.0.1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_ipv4_address_spec_for_multicast() {
        let spec = SocketAddressSpec {
            ip_address: "239.1.1.1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert multicast address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_ipv4_address_spec_for_incomplete_address() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_without_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "2001:4860:4860::8888".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(
                    addr.ip(),
                    Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)
                );
                assert_eq!(addr.port(), 179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_with_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "2001:4860:4860::8888".to_string(),
            port_number: 1179,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(
                    addr.ip(),
                    Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)
                );
                assert_eq!(addr.port(), 1179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_ipv6_address_spec_for_localhost() {
        let spec = SocketAddressSpec {
            ip_address: "::1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_ipv6_address_spec_for_multicast() {
        let spec = SocketAddressSpec {
            ip_address: "ff02::1".to_string(),
            port_number: 0,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert multicast address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_peer_config_file() {
        let peer_spec = PeerConfigFile {
            peer_as: AsNumber::Small(1234),
            peer_name: "Some peer".to_string(),
            peer_address: SocketAddressSpec {
                ip_address: "192.168.1.1".to_string(),
                port_number: 0,
            },
            local_address: SocketAddressSpec {
                ip_address: "192.168.2.2".to_string(),
                port_number: 0,
            },
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
            peer_address: SocketAddressSpec {
                ip_address: "192.168.1.1".to_string(),
                port_number: 0,
            },
            local_address: SocketAddressSpec {
                ip_address: "127.0.0.1".to_string(),
                port_number: 0,
            },
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
            peer_address: SocketAddressSpec {
                ip_address: "127.0.0.1".to_string(),
                port_number: 0,
            },
            local_address: SocketAddressSpec {
                ip_address: "192.168.2.2".to_string(),
                port_number: 0,
            },
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
            peer_address: SocketAddressSpec {
                ip_address: "192.168.1.1".to_string(),
                port_number: 0,
            },
            local_address: SocketAddressSpec {
                ip_address: "192.168.1.1".to_string(),
                port_number: 0,
            },
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
                peer_address: SocketAddressSpec {
                    ip_address: "192.168.1.1".to_string(),
                    port_number: 0,
                },
                local_address: SocketAddressSpec {
                    ip_address: "192.168.2.2".to_string(),
                    port_number: 0,
                },
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
                peer_address: SocketAddressSpec {
                    ip_address: "192.168.1.1".to_string(),
                    port_number: 0,
                },
                local_address: SocketAddressSpec {
                    ip_address: "192.168.1.1".to_string(),
                    port_number: 0,
                },
            }],
        };

        match TryInto::<RouterConfiguration>::try_into(router_spec) {
            Ok(mut router_config) => panic!("Should not convert to router configuration"),
            Err(err) => {}
        }
    }
}
