use super::*;
use crate::data::ServiceStatus;
use anyhow::Result;
use std::fs::OpenOptions;
use std::sync::Arc;

const DHCPD_LEASE_DB: &'static str = "/var/lib/dhcp/dhcpd.leases";

config_file! { DhcpConfigTemplate("dhcpd.conf.hbs") => "/etc/dhcp/dhcpd.conf" }

#[derive(Debug, Clone)]
pub struct DhcpService {
    process: Arc<ProcessManager<DhcpMeta>>,
    state_manager: StateManager,
}

#[derive(Debug, Clone)]
pub struct DhcpMeta;
impl ProcessService for DhcpMeta {
    const SERVICE_NAME: &'static str = "DHCP";
    const COMMAND: &'static str = "dhcpd";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> &[&str] {
        &[
            "-cf",
            DhcpConfigTemplate::FILE_PATH,
            "-lf",
            DHCPD_LEASE_DB,
            "-f",
            "--no-pid",
        ]
    }
}

impl DhcpService {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            process: Arc::new(ProcessManager::new(DhcpMeta, state_manager).await?),
        })
    }

    pub async fn status(&self) -> Result<ServiceStatus> {
        Ok(ServiceStatus {
            enabled: self.state_manager.config.dhcp.enabled,
            state: self.process.current_state().await?,
            detail: Default::default(),
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dhcp.enabled {
            DhcpConfigTemplate::write(self.state_manager.config.clone()).await?;
            OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(DHCPD_LEASE_DB)
                .map(|_| "")
                .unwrap_or_default();
            self.process.clone().spawn().await?;
        } else {
            info!("DHCP service is disabled");
        }
        Ok(())
    }
}
