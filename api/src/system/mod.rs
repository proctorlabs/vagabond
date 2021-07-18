use crate::state::StateManager;
use crate::util::run_command;
use crate::{config::VagabondConfig, data::Interfaces, services::DhcpClient};
use anyhow::{anyhow, Result};
use nix::unistd;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::spawn_blocking;
pub use wireguard::*;

mod ctls;
mod interfaces;
mod routing;
mod wireguard;
mod network;

#[derive(Debug)]
pub struct SystemInfo {
    state: StateManager,
    pub is_root: bool,
    dhcp_clients: Arc<RwLock<HashMap<String, DhcpClient>>>,
    wireguard: Wireguard,
}

#[derive(Debug, Clone, Deref)]
pub struct SystemManager(Arc<SystemInfo>);

impl SystemManager {
    pub fn new(state: StateManager) -> Self {
        let is_root = unistd::geteuid().is_root();
        if !is_root {
            warn!(
                "NOTICE: Some functionality will be disabled as this app is not running as root!"
            );
        }

        SystemManager(Arc::new(SystemInfo {
            dhcp_clients: Default::default(),
            wireguard: Wireguard(state.clone()),
            state,
            is_root,
        }))
    }

    pub fn get_interfaces(&self) -> Result<Interfaces> {
        let mut ifaces = interfaces::get_interfaces()?;
        let mut result: Interfaces = Default::default();
        let mut if_whitelist = self.state.config.network.interfaces();
        if self.state.config.wireguard.enabled {
            if_whitelist.push(self.state.config.wireguard.interface.to_string());
        }
        for (k, v) in ifaces.drain() {
            if if_whitelist.contains(&k) {
                result.insert(k, v);
            }
        }
        Ok(result)
    }

    pub fn get_all_interface_names(&self) -> Result<Vec<String>> {
        let ifaces = interfaces::get_interfaces()?;
        Ok(ifaces.iter().map(|(_, v)| v.name.to_owned()).collect())
    }

    pub fn setup_sysctl(&self) -> Result<()> {
        if self.is_root {
            ctls::set_sysctls()?;
        } else {
            warn!("Cannot set sysctl when not root!");
        }
        Ok(())
    }

    pub async fn setup_iptables(&self) -> Result<()> {
        if self.is_root {
            let cfg = self.state.config.clone();
            spawn_blocking(move || routing::setup_routes(&cfg))
                .await?
                .map_err(|e| anyhow!(format!("{}", e)))?;
            Ok(())
        } else {
            warn!("IPTables cannot be setup as a non-root user!");
            Ok(())
        }
    }

    pub async fn setup_interfaces(&self) -> Result<()> {
        for wan in self.state.config.network.wans.iter() {
            if wan.is_dhcp() {
                let mut dhcp_map = self.dhcp_clients.write().await;
                let iface = wan.interface_name();
                let client = DhcpClient::new(self.state.clone(), iface.clone()).await?;
                client.spawn().await?;
                dhcp_map.insert(iface, client);
            }
        }

        if self.state.config.network.lan.enabled {
            info!("Configuring LAN interface");
            let ifname = self.state.config.network.lan.interface.as_str();
            let addr = self.state.config.network.lan.address.to_string();
            let prefix = self.state.config.network.lan.subnet.prefix_len();
            let addr = format!("{}/{}", addr, prefix);
            let addr_str = addr.as_str();
            run_command("ip", ["link", "set", ifname, "up"]).await?;
            run_command("ip", ["addr", "change", addr_str, "dev", ifname]).await?;
        }

        if self.state.config.network.wlan.enabled {
            info!("Configuring WLAN interface");
            let ifname = self.state.config.network.wlan.interface.as_str();
            let addr = self.state.config.network.wlan.address.to_string();
            let prefix = self.state.config.network.wlan.subnet.prefix_len();
            let addr = format!("{}/{}", addr, prefix);
            let addr_str = addr.as_str();
            run_command("ip", ["link", "set", ifname, "up"]).await?;
            run_command("ip", ["addr", "change", addr_str, "dev", ifname]).await?;
        }

        if self.state.config.wireguard.enabled {
            self.wireguard.setup().await?;
        }
        Ok(())
    }

    pub async fn dhcp_renew(&self, interface_name: &str) -> Result<()> {
        let clients = self.dhcp_clients.read().await;
        let iface = clients.get(interface_name);
        match iface {
            Some(iface) => iface.renew().await,
            None => Err(anyhow::anyhow!("Interface {} not found!", interface_name)),
        }
    }

    pub async fn dhcp_release(&self, interface_name: &str) -> Result<()> {
        let clients = self.dhcp_clients.read().await;
        let iface = clients.get(interface_name);
        match iface {
            Some(iface) => iface.release().await,
            None => Err(anyhow::anyhow!("Interface {} not found!", interface_name)),
        }
    }
}
