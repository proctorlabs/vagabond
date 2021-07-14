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
            result.push(self.lan.interface.clone())
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
