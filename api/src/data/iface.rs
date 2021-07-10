// use super::*;
use ipnet::{Ipv4Net, Ipv6Net};
use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

pub type Interfaces = HashMap<String, Interface>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    pub name: String,
    pub up: bool,
    pub addresses: Vec<InterfaceAddress>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum InterfaceAddress {
    Ipv4 { address: Ipv4Addr, subnet: Ipv4Net },
    Ipv6 { address: Ipv6Addr, subnet: Ipv6Net },
    Mac { address: [u8; 6] },
}
