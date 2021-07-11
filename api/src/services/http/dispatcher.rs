use crate::{app::Vagabond, data::*};
use anyhow::Result;
use futures::{stream::SplitSink, SinkExt};
use warp::ws::{Message, WebSocket};

async fn send(
    msg: &WebsocketTxMessage,
    tx: &mut SplitSink<WebSocket, Message>,
) -> anyhow::Result<()> {
    let msg = warp::ws::Message::text(serde_json::to_string(&msg)?);
    tx.send(msg).await?;
    Ok(())
}

impl WebsocketRxMessage {
    pub async fn dispatch(
        &self,
        app: &Vagabond,
        tx: &mut SplitSink<WebSocket, Message>,
    ) -> Result<()> {
        match self {
            &WebsocketRxMessage::WifiScan => {
                let response = WebsocketTxMessage::WifiScan(app.iwd.get_wifi_networks().await?);
                send(&response, tx).await?
            }
            &WebsocketRxMessage::WifiStatus => {
                let response = WebsocketTxMessage::WifiStatus(app.iwd.get_wifi_device().await?);
                send(&response, tx).await?
            }
            &WebsocketRxMessage::ListInterfaces => {
                let response = WebsocketTxMessage::Interfaces(app.system.get_interfaces()?);
                send(&response, tx).await?
            }
            &WebsocketRxMessage::WifiConnect(ref params) => app.iwd.connect(params).await?,
            &WebsocketRxMessage::WifiDisconnect => app.iwd.disconnect().await?,
            &WebsocketRxMessage::GetStatus => {
                let hostapd = app.hostapd.status().await?;
                let unbound = app.dns.status().await?;
                let dhcpd = app.dhcp.status().await?;
                let response = WebsocketTxMessage::Status {
                    hostapd,
                    unbound,
                    dhcpd,
                };
                send(&response, tx).await?
            }
            WebsocketRxMessage::DhcpRenew(iface) => {
                app.system.dhcp_renew(iface).await?;
            }
            WebsocketRxMessage::DhcpRelease(iface) => {
                app.system.dhcp_release(iface).await?;
            }
        }
        Ok(())
    }
}
