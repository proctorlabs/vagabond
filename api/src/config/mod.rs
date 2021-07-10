use anyhow::Result;
use network::NetworkWan;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

mod dhcp;
mod dns;
mod network;
mod wireguard;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub dns: dns::DNSConfig,
    pub dhcp: dhcp::DHCPConfig,
    pub wireguard: wireguard::WireguardConfig,
    pub network: network::NetworkConfig,
}

#[derive(Clone, Debug, Deref)]
pub struct VagabondConfig(Arc<Config>);

impl VagabondConfig {
    pub async fn from_file(file: PathBuf) -> Result<Self> {
        let mut f = File::open(file).await?;
        let mut contents = vec![];
        f.read_to_end(&mut contents).await?;
        let result = toml::from_slice(&contents)?;
        trace!("Parsed configuration: {:?}", result);
        Ok(VagabondConfig(Arc::new(result)))
    }

    pub fn external_interfaces(&self) -> Vec<String> {
        let mut result = vec![];
        for wan in &self.network.wans {
            match wan {
                NetworkWan::DHCP { interface }
                | NetworkWan::Unmanaged { interface }
                | NetworkWan::WLAN { interface } => {
                    result.push(interface.clone());
                }
            }
        }
        result
    }
}
