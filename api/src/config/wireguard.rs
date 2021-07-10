use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct WireguardConfig {
    pub enabled: bool,
    pub interface: String,
    pub address: IpAddr,
    pub private_key: String,

    #[serde(rename = "peer")]
    pub peers: Vec<WireguardPeer>,
}

impl Default for WireguardConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interface: "wg0".into(),
            address: Ipv4Addr::new(0, 0, 0, 0).into(),
            private_key: Default::default(),
            peers: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct WireguardPeer {
    pub public_key: String,
    pub endpoint: IpAddr,
    pub endpoint_port: u16,
    pub allowed_ips: String,
}
