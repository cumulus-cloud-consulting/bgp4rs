use std::net::IpAddr;
use log::{info,error};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use crate::shared::error::Error::UnspecifiedError;
use crate::shared::prelude::Result;
use async_trait::async_trait;


#[async_trait]
pub trait LocalAddressMatcher {
    async fn is_local_address(&self, ip_address: &IpAddr) -> bool;
}

pub struct HostInterfacesLocalAddressMatcher {
    local_intf_addresses: Vec<IpAddr>
}


#[async_trait]
impl LocalAddressMatcher for HostInterfacesLocalAddressMatcher {
    async fn is_local_address(&self, ip_address: &IpAddr) -> bool {
        self.local_intf_addresses.contains(&ip_address)
    }
}

impl HostInterfacesLocalAddressMatcher {
    pub fn new() -> Result<Box<dyn LocalAddressMatcher>> {
        match NetworkInterface::show() {
            Ok(network_interfaces   ) => {
                let mut hosts_addrs : Vec<IpAddr> = Vec::new();

                for ni in network_interfaces {
                    let if_name = &ni.name;

                    for host_addr in ni.addr{
                        let if_addr = host_addr.ip();

                        info!("Find network address {} for interface {}", &if_addr, &if_name);

                        hosts_addrs.push(if_addr);
                    }
                }

                Ok(Box::new(HostInterfacesLocalAddressMatcher {
                    local_intf_addresses: hosts_addrs,
                }))
            },
            Err(error) => {
                error!("Cannot enumerate host network interfaces: {}", error);

                Err(UnspecifiedError(anyhow::Error::new(error)))
            }
        }
    }
}