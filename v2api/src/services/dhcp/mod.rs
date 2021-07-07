use super::*;
use anyhow::Result;
use std::sync::Arc;

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
    const COMMAND: &'static str = "bash";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> &[&str] {
        &[
            "-c",
            "echo 'Hi from dhcp!!' && sleep 5 && echo 'yo!' && sleep 2",
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

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dhcp.enabled {
            // DhcpConfigTemplate::write(self.config.clone()).await?;
            self.process.clone().spawn().await?;
        } else {
            info!("DHCP service is disabled");
        }
        Ok(())
    }
}
