use super::*;
use anyhow::Result;
use std::sync::Arc;

config_file! { HostapdConfigurationTemplate("hostapd.conf.hbs") => "/data/hostapd/hostapd.conf" }

#[derive(Debug, Clone)]
pub struct HostapdService {
    process: Arc<ProcessManager<HostapdMeta>>,
    state_manager: StateManager,
}

#[derive(Debug, Clone)]
pub struct HostapdMeta;
impl ProcessService for HostapdMeta {
    const SERVICE_NAME: &'static str = "hostapd";
    const COMMAND: &'static str = "bash";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> &[&str] {
        &["-c", "echo 'Hey!' && sleep 5 && echo 'yo!' && sleep 2"]
    }
}

impl HostapdService {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            process: Arc::new(ProcessManager::new(HostapdMeta, state_manager).await?),
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dns.enabled {
            // HostapdConfigurationTemplate::write(self.config.clone()).await?;
            self.process.clone().spawn().await?;
        } else {
            info!("DNS service is disabled");
        }
        Ok(())
    }
}
