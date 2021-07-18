// use cidr_utils::cidr::*;
use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct NetworkConfig {
    pub domain: String,
    pub manage_routes: bool,
    pub lan: NetworkLan,
    pub wlan: NetworkWlan,

    #[serde(rename = "wan")]
    pub wans: Vec<NetworkWan>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            domain: "vagabond.lan".into(),
            manage_routes: true,
            lan: Default::default(),
            wlan: Default::default(),
            wans: vec![NetworkWan::DHCP {
                interface: "eth0".into(),
            }],
        }
    }
}

impl NetworkConfig {
    pub const PUBLIC_SUBNETS: &'static str = "0.0.0.0/5, 8.0.0.0/7, 11.0.0.0/8, 12.0.0.0/6, \
 16.0.0.0/4, 32.0.0.0/3, 64.0.0.0/2, 128.0.0.0/3, 160.0.0.0/5, 168.0.0.0/6, 172.0.0.0/12, \
 172.32.0.0/11, 172.64.0.0/10, 172.128.0.0/9, 173.0.0.0/8, 174.0.0.0/7, 176.0.0.0/4, \
 192.0.0.0/9, 192.128.0.0/11, 192.160.0.0/13, 192.169.0.0/16, 192.170.0.0/15, 192.172.0.0/14, \
 192.176.0.0/12, 192.192.0.0/10, 193.0.0.0/8, 194.0.0.0/7, 196.0.0.0/6, 200.0.0.0/5, 208.0.0.0/4";

    pub fn wifi_wan_interfaces(&self) -> Vec<String> {
        self.wans
            .iter()
            .filter_map(|wan| {
                if wan.is_wifi() {
                    Some(wan.interface_name())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn wan_interfaces(&self) -> Vec<String> {
        self.wans.iter().map(|i| i.interface_name()).collect()
    }

    pub fn local_interfaces(&self) -> Vec<String> {
        let mut result = vec![];
        if self.lan.enabled {
            result.push(self.lan.interface.clone())
        }
        if self.wlan.enabled {
            result.push(self.wlan.interface.clone())
        }
        result
    }

    pub fn interfaces(&self) -> Vec<String> {
        let mut result = self.wan_interfaces();
        result.append(&mut self.local_interfaces());
        result
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, IsVariant)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum NetworkWan {
    #[serde(rename = "dhcp")]
    DHCP { interface: String },
    #[serde(rename = "wifi")]
    Wifi { interface: String },
    #[serde(rename = "unmanaged")]
    Unmanaged { interface: String },
}

impl NetworkWan {
    pub fn interface_name(&self) -> String {
        match &self {
            &NetworkWan::DHCP { interface }
            | &NetworkWan::Wifi { interface }
            | &NetworkWan::Unmanaged { interface } => interface.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NetworkLan {
    pub enabled: bool,
    pub interface: String,
    pub subnet: IpNet,
    pub address: IpAddr,
}

impl Default for NetworkLan {
    fn default() -> Self {
        Self {
            enabled: true,
            interface: "eth1".into(),
            subnet: "192.168.1.0/24".parse().unwrap(),
            address: Ipv4Addr::new(192, 168, 1, 1).into(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct NetworkWlan {
    pub enabled: bool,
    pub interface: String,
    pub subnet: IpNet,
    pub address: IpAddr,
    pub channel: u16,
    pub hostapd_config: String,
    pub ssid: String,
}

impl Default for NetworkWlan {
    fn default() -> Self {
        Self {
            enabled: false,
            interface: "wlan0".into(),
            subnet: "192.168.2.0/24".parse().unwrap(),
            address: Ipv4Addr::new(192, 168, 2, 1).into(),
            channel: 1,
            hostapd_config: Default::default(),
            ssid: "vagabond".into(),
        }
    }
}
