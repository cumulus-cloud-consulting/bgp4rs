// Copyright 2025 Rainer Bieniek <Rainer.Bieniek@cumulus-cloud-consulting.de>
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
//
use crate::shared::error::Error::UnspecifiedError;
use crate::shared::prelude::Result;
use async_trait::async_trait;
use log::{error, info};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use std::net::{IpAddr, SocketAddr};

/// Trait boundary to define the interface of a capability to validate if a given [`IpAddr`]
/// or [`SocketAddr`] is handled by a host network interfaces.
///
/// Using this capability should prevent from attempts to bind to listen address when this
/// address is not available in the host system configuration
///
#[async_trait]
pub trait LocalAddressMatcher {
    /// Validate if a given [`IpAddr`] matches any network interface on the host
    async fn is_local_address(&self, ip_address: &IpAddr) -> bool;

    /// Validate if a given [`SocketAddr`] matches any network interface on the host
    async fn is_local_socket_address(&self, socket_address: &SocketAddr) -> bool;
}

/// Concreate implementation of the [`LocalAddressMatcher`] trait.
///
/// The trait boundary is used to hide the internals of this implementation
///
pub struct HostInterfacesLocalAddressMatcher {
    local_intf_addresses: Vec<IpAddr>,
}

#[async_trait]
impl LocalAddressMatcher for HostInterfacesLocalAddressMatcher {
    async fn is_local_address(&self, ip_address: &IpAddr) -> bool {
        self.local_intf_addresses.contains(&ip_address)
    }

    async fn is_local_socket_address(&self, socket_address: &SocketAddr) -> bool {
        self.is_local_address(&socket_address.ip()).await
    }
}

impl HostInterfacesLocalAddressMatcher {
    /// Create a new instance and return it as a reference to the trait object
    pub fn new() -> Result<Box<dyn LocalAddressMatcher + Send + Sync>> {
        match NetworkInterface::show() {
            Ok(network_interfaces) => {
                let mut hosts_addrs: Vec<IpAddr> = Vec::new();

                for ni in network_interfaces {
                    let if_name = &ni.name;

                    for host_addr in ni.addr {
                        let if_addr = host_addr.ip();

                        info!(
                            "Find network address {} for interface {}",
                            &if_addr, &if_name
                        );

                        hosts_addrs.push(if_addr);
                    }
                }

                Ok(Box::new(HostInterfacesLocalAddressMatcher {
                    local_intf_addresses: hosts_addrs,
                }))
            }
            Err(error) => {
                error!("Cannot enumerate host network interfaces: {}", error);

                Err(UnspecifiedError(anyhow::Error::new(error)))
            }
        }
    }
}
