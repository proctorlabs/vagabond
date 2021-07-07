use super::*;
use anyhow::Result;
use std::sync::Arc;

config_file! { UnboundConfigTemplate("unbound.conf.hbs") => "/etc/unbound/unbound.conf" }

#[derive(Debug, Clone)]
pub struct DnsService {
    process: Arc<ProcessManager<DnsMeta>>,
    state_manager: StateManager,
}

#[derive(Debug, Clone)]
pub struct DnsMeta;
impl ProcessService for DnsMeta {
    const SERVICE_NAME: &'static str = "DNS";
    const COMMAND: &'static str = "bash";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> &[&str] {
        &["-c", "echo 'Hey!' && sleep 5 && echo 'yo!' && sleep 2"]
    }
}

impl DnsService {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            process: Arc::new(ProcessManager::new(DnsMeta, state_manager).await?),
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dns.enabled {
            // UnboundConfigTemplate::write(self.config.clone()).await?;
            self.process.clone().spawn().await?;
        } else {
            info!("DNS service is disabled");
        }
        Ok(())
    }
}
