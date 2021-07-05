use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct DHCPConfig {
    pub enabled: bool,
    pub extra_config: String,
    pub lan: DHCPNetwork,
    pub wlan: DHCPNetwork,
}

impl Default for DHCPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            extra_config: Default::default(),
            lan: Default::default(),
            wlan: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct DHCPNetwork {
    pub range: DHCPRange,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct DHCPRange {
    pub start: IpAddr,
    pub end: IpAddr,
}

impl Default for DHCPRange {
    fn default() -> Self {
        Self {
            start: Ipv4Addr::new(192, 168, 1, 100).into(),
            end: Ipv4Addr::new(192, 168, 1, 199).into(),
        }
    }
}
