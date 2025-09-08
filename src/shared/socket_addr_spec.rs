use crate::shared::error::Error::{InvalidIpAddressError, ParseIpAddressError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SocketAddressSpec {
    pub ip_address: String,
    pub port_number: Option<u16>,
}

impl SocketAddressSpec {
    pub fn with_default_port(self, def_port_number: u16) -> Self {
        match self.port_number {
            Some(port_number) => self,
            None => SocketAddressSpec {
                ip_address: self.ip_address,
                port_number: Some(def_port_number),
            }
        }
    }
}

impl Display for SocketAddressSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IP Address {}, port number{}",
            &self.ip_address,
            &self.port_number.unwrap_or(0)
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

                Ok(SocketAddr::new(addr, self.port_number.unwrap_or(0)))
            }
            Err(err) => Err(ParseIpAddressError(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn should_convert_ipv4_address_spec_without_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: None,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
                assert_eq!(addr.port(), 0);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv4_address_spec_with_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: Some(1179),
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
            port_number: None,
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
            port_number: None,
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
            port_number: None,
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
            port_number: None,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(
                    addr.ip(),
                    Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888)
                );
                assert_eq!(addr.port(), 0);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_with_port_number() {
        let spec = SocketAddressSpec {
            ip_address: "2001:4860:4860::8888".to_string(),
            port_number: Some(1179),
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
            port_number: None,
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
            port_number: None,
        };

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert multicast address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv4_address_spec_without_port_number_with_default_port() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: None,
        }.with_default_port(179);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
                assert_eq!(addr.port(), 179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_without_port_number_with_default_port() {
        let spec = SocketAddressSpec {
            ip_address: "2001:4860:4860::8888".to_string(),
            port_number: None,
        }.with_default_port(179);

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
    fn should_convert_ipv4_address_spec_with_port_number_without_default_port() {
        let spec = SocketAddressSpec {
            ip_address: "192.168.1.1".to_string(),
            port_number: Some(1179),
        }.with_default_port(179);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(192, 168, 1, 1));
                assert_eq!(addr.port(), 1179);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_with_port_number_without_default_port() {
        let spec = SocketAddressSpec {
            ip_address: "2001:4860:4860::8888".to_string(),
            port_number: Some(1179),
        }.with_default_port(179);

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
}