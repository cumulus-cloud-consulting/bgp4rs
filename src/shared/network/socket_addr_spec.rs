// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::error::Error::{InvalidIpAddressError, ParseIpAddressError};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SocketAddressSpec {
    ip_address: String,
    port_number: Option<u16>,
    #[serde(skip)]
    localhost_allowed: bool,
}

impl SocketAddressSpec {
    #[allow(dead_code)]
    pub fn new(ip_address: &str, port_number: &Option<u16>) -> Self {
        SocketAddressSpec {
            ip_address: ip_address.to_string(),
            port_number: port_number.clone(),
            localhost_allowed: false,
        }
    }

    pub fn new_allowing_localhost(ip_address: &str, port_number: &Option<u16>) -> Self {
        SocketAddressSpec {
            ip_address: ip_address.to_string(),
            port_number: port_number.clone(),
            localhost_allowed: true,
        }
    }

    pub fn with_default_port(self, def_port_number: u16) -> Self {
        match self.port_number {
            Some(_port_number) => self,
            None => SocketAddressSpec {
                ip_address: self.ip_address,
                port_number: Some(def_port_number),
                localhost_allowed: self.localhost_allowed,
            },
        }
    }
}

impl Display for SocketAddressSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "IP Address {}, port number{}, localhost allowed {}",
            &self.ip_address,
            &self.port_number.unwrap_or(0),
            &self.localhost_allowed
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
                if (!self.localhost_allowed && addr.is_loopback()) || addr.is_multicast() {
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
        let spec = SocketAddressSpec::new(&"192.168.1.1", &None);

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
        let spec = SocketAddressSpec::new(&"192.168.1.1", &Some(1179));

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
        let spec = SocketAddressSpec::new(&"127.0.0.1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv4_address_spec_for_allowed_localhost() {
        let spec = SocketAddressSpec::new_allowing_localhost(&"127.0.0.1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv4Addr::new(127, 0, 0, 1));
                assert_eq!(addr.port(), 0);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_ipv4_address_spec_for_multicast() {
        let spec = SocketAddressSpec::new(&"239.1.1.1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert multicast address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_not_convert_ipv4_address_spec_for_incomplete_address() {
        let spec = SocketAddressSpec::new(&"192.168.1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_without_port_number() {
        let spec = SocketAddressSpec::new(&"2001:4860:4860::8888", &None);

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
        let spec = SocketAddressSpec::new(&"2001:4860:4860::8888", &Some(1179));

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
        let spec = SocketAddressSpec::new(&"::1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert localhost address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv6_address_spec_for_allowed_localhost() {
        let spec = SocketAddressSpec::new_allowing_localhost(&"::1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                assert_eq!(addr.ip(), Ipv6Addr::new(0x0, 0x0, 0x0, 0, 0, 0, 0, 0x1));
                assert_eq!(addr.port(), 0);
            }
            Err(err) => panic!("Received error: {err}"),
        }
    }

    #[test]
    fn should_not_convert_ipv6_address_spec_for_multicast() {
        let spec = SocketAddressSpec::new(&"ff02::1", &None);

        match TryInto::<SocketAddr>::try_into(spec) {
            Ok(addr) => {
                panic!("Should not have been able to convert multicast address.");
            }
            Err(err) => {}
        }
    }

    #[test]
    fn should_convert_ipv4_address_spec_without_port_number_with_default_port() {
        let spec = SocketAddressSpec::new(&"192.168.1.1", &None).with_default_port(179);

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
        let spec = SocketAddressSpec::new(&"2001:4860:4860::8888", &None).with_default_port(179);

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
        let spec = SocketAddressSpec::new(&"192.168.1.1", &Some(1179)).with_default_port(179);

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
        let spec =
            SocketAddressSpec::new(&"2001:4860:4860::8888", &Some(1179)).with_default_port(179);

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
