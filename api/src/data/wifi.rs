// use super::*;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct WifiDevice {
    pub name: String,
    pub phy: String,
    pub state: WifiState,
    pub address: String,
    pub powered: bool,
    pub scanning: bool,
    pub mode: WifiMode,
    pub supported_modes: Vec<WifiMode>,
    pub model: String,
    pub vendor: String,
    pub connected_network: Option<WifiNetwork>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct WifiNetwork {
    pub ssid: String,
    pub security: WifiSecurity,
    pub signal: i16,
    pub known: bool,
    pub interface: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "snake_case", deny_unknown_fields, untagged)]
pub enum ConnectionParameters {
    Ssid { ssid: String },
    PresharedKey { ssid: String, psk: String },
}

#[derive(Debug, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WifiSecurity {
    Open,
    Wep,
    Psk,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WifiMode {
    AdHoc,
    Station,
    AccessPoint,
    None,
    Other(String),
}

impl From<&str> for WifiMode {
    fn from(mode: &str) -> Self {
        match mode.to_lowercase().as_str() {
            "station" => WifiMode::Station,
            "ad-hoc" | "adhoc" => WifiMode::AdHoc,
            "ap" | "accesspoint" | "access point" | "access-point" => WifiMode::AccessPoint,
            "" => WifiMode::None,
            o => WifiMode::Other(o.to_string()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, IsVariant, PartialEq, PartialOrd, Eq, Ord)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum WifiState {
    Disconnected,
    Connected,
}

impl From<&str> for WifiState {
    fn from(state: &str) -> Self {
        match state.to_lowercase().as_str() {
            "connected" => WifiState::Connected,
            _ => WifiState::Disconnected,
        }
    }
}

impl From<String> for WifiSecurity {
    fn from(sec: String) -> Self {
        match sec.to_lowercase().as_str() {
            "psk" => Self::Psk,
            "wep" => Self::Wep,
            "open" => Self::Open,
            _ => Self::Other(sec),
        }
    }
}
