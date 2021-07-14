use super::*;
use crate::data::ServiceStatus;
use anyhow::Result;
use std::sync::Arc;

config_file! { DhcpLanConfigTemplate("udhcpd.lan.conf.hbs") => "/etc/udhcpd.lan.conf" }
config_file! { DhcpWlanConfigTemplate("udhcpd.wlan.conf.hbs") => "/etc/udhcpd.wlan.conf" }

#[derive(Debug, Clone)]
pub struct DhcpLanServerProcess;
impl ProcessService for DhcpLanServerProcess {
    const SERVICE_NAME: &'static str = "DHCP LAN Server";
    const COMMAND: &'static str = "udhcpd";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> Vec<String> {
        vec!["-f".into(), DhcpLanConfigTemplate::FILE_PATH.into()]
    }
}

#[derive(Debug, Clone)]
pub struct DhcpWlanServerProcess;
impl ProcessService for DhcpWlanServerProcess {
    const SERVICE_NAME: &'static str = "DHCP WLAN Server";
    const COMMAND: &'static str = "udhcpd";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> Vec<String> {
        vec!["-f".into(), DhcpWlanConfigTemplate::FILE_PATH.into()]
    }
}

#[derive(Debug, Clone)]
pub struct DhcpServer {
    lan_service: Arc<ProcessManager<DhcpLanServerProcess>>,
    wlan_service: Arc<ProcessManager<DhcpWlanServerProcess>>,
    state_manager: StateManager,
}

impl DhcpServer {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            lan_service: Arc::new(
                ProcessManager::new(DhcpLanServerProcess, state_manager.clone()).await?,
            ),
            wlan_service: Arc::new(
                ProcessManager::new(DhcpWlanServerProcess, state_manager).await?,
            ),
        })
    }

    pub async fn status(&self) -> Result<ServiceStatus> {
        Ok(ServiceStatus {
            enabled: self.state_manager.config.dhcp.enabled,
            state: self.lan_service.current_state().await?,
            detail: Default::default(),
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dhcp.enabled {
            if self.state_manager.config.network.lan.enabled {
                DhcpLanConfigTemplate::write(self.state_manager.config.clone()).await?;
                self.lan_service.clone().spawn().await?;
            }
            if self.state_manager.config.network.wlan.enabled {
                DhcpWlanConfigTemplate::write(self.state_manager.config.clone()).await?;
                self.wlan_service.clone().spawn().await?;
            }
        }
        Ok(())
    }
}
