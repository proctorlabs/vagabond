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
    pub async fn dispatch(&self) -> Result<()> {
        match self {
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
}

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