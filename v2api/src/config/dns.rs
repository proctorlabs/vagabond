use std::net::{IpAddr, Ipv4Addr};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default, deny_unknown_fields)]
pub struct DNSConfig {
    pub enabled: bool,
    pub block_malicious: bool,
    pub port: u16,
    pub servers: Vec<IpAddr>,
    pub unbound_options: String,
}

impl Default for DNSConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            block_malicious: true,
            port: 53,
            servers: vec![
                Ipv4Addr::new(1, 1, 1, 1).into(),
                Ipv4Addr::new(1, 0, 0, 1).into(),
            ],
            unbound_options: Default::default(),
        }
    }
}
