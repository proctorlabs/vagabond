use super::*;

#[derive(Debug, Serialize, Deserialize, IsVariant)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WebsocketRxMessage {
    WifiScan,
    WifiStatus,
    WifiConnect(ConnectionParameters),
    WifiDisconnect,
    GetStatus,
    ListInterfaces,
}

#[derive(Debug, Serialize, Deserialize, IsVariant)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum WebsocketTxMessage {
    Status {
        hostapd: ServiceStatus,
        unbound: ServiceStatus,
        dhcpd: ServiceStatus,
    },
    Interfaces(Interfaces),
    WifiStatus(WifiDevice),
    WifiScan(Vec<WifiNetwork>),
    Error(String),
}
