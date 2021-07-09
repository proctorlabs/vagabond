use anyhow::Result;
use ipnet::{Ipv4Net, Ipv6Net};
use nix::sys::socket::InetAddr;
use nix::{ifaddrs, net::if_::InterfaceFlags, sys::socket::SockAddr};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    name: String,
    up: bool,
    addresses: Vec<InterfaceAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
pub enum InterfaceAddress {
    Ipv4 { address: Ipv4Addr, subnet: Ipv4Net },
    Ipv6 { address: Ipv6Addr, subnet: Ipv6Net },
    Mac { address: [u8; 6] },
}

pub fn get_interfaces() -> Result<HashMap<String, Interface>> {
    let mut result: HashMap<String, Interface> = HashMap::new();
    for iface in ifaddrs::getifaddrs()?.into_iter() {
        if !result.contains_key(&iface.interface_name) {
            result.insert(
                iface.interface_name.clone(),
                Interface {
                    name: iface.interface_name.clone(),
                    up: iface.flags.contains(InterfaceFlags::IFF_UP),
                    addresses: vec![],
                },
            );
        }

        let if_ = result.get_mut(&iface.interface_name).unwrap();
        match (&iface.address, &iface.netmask) {
            (
                Some(SockAddr::Inet(InetAddr::V4(addr))),
                Some(SockAddr::Inet(InetAddr::V4(nmask))),
            ) => {
                let ipv4addr: Ipv4Addr = addr.sin_addr.s_addr.to_be().into();
                let ipv4nmask: u8 = nmask.sin_addr.s_addr.count_ones() as u8;
                let ifa = InterfaceAddress::Ipv4 {
                    address: ipv4addr,
                    subnet: Ipv4Net::new(ipv4addr, ipv4nmask).unwrap().trunc(),
                };
                if_.addresses.push(ifa);
            }
            (
                Some(SockAddr::Inet(InetAddr::V6(addr))),
                Some(SockAddr::Inet(InetAddr::V6(nmask))),
            ) => {
                let ipv6addr: Ipv6Addr = addr.sin6_addr.s6_addr.into();
                let ipv6nmask: u8 = nmask
                    .sin6_addr
                    .s6_addr
                    .iter()
                    .fold(0 as u8, |f, v| f + v.count_ones() as u8);
                let ifa = InterfaceAddress::Ipv6 {
                    address: ipv6addr,
                    subnet: Ipv6Net::new(ipv6addr, ipv6nmask).unwrap().trunc(),
                };
                if_.addresses.push(ifa);
            }
            (Some(SockAddr::Link(ll)), _) => {
                let b = ll.0.sll_addr;
                let mac: [u8; 6] = [b[0], b[1], b[2], b[3], b[4], b[5]];
                if_.addresses.push(InterfaceAddress::Mac { address: mac });
            }
            _ => {}
        };
    }
    Ok(result)
}
