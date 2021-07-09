use crate::app::Vagabond;
use anyhow::Result;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WebsocketRxMessage {
    WifiScan,
    WifiStatus,
    WifiConnect { ssid: String, psk: String },
    WifiDisconnect,
    GetStatus,
    ListInterfaces,
}

impl WebsocketRxMessage {
    pub async fn dispatch(&self, app: &Vagabond) -> Result<()> {
        match self {
            &WebsocketRxMessage::WifiScan => {
                app.iwd.run_test().await?;
            }
            m => {
                info!("No handler for {:?}", m);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WebsocketTxMessage {
    Status {
        hostapd: HashMap<String, String>,
        iptables: HashMap<String, String>,
        wireguard: HashMap<String, String>,
        interfaces: HashMap<String, String>,
        unbound: HashMap<String, String>,
        dhcpd: HashMap<String, String>,
    },
    Interfaces(InterfaceMessage),
    WifiStatus(Vec<WifiNetwork>),
    Error(String),
}

// "type": self.interface_type,
// "running": self._running,
// "interface": self._ip_addr,

#[derive(Debug, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum InterfaceMessage {
    Wifi {
        name: String,
        address: String,
        mode: String,
        state: String,
        ip: String,
        ssid: Option<String>,
        rssi: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct WifiNetwork {
    ssid: String,
    security: String,
    signal: i16,
    known: bool,
}
