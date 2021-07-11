use super::*;
use crate::data::ServiceStatus;
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
    const SERVICE_NAME: &'static str = "DNS Server";
    const COMMAND: &'static str = "unbound";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> Vec<String> {
        vec![
            "-d".into(),
            "-p".into(),
            "-c".into(),
            UnboundConfigTemplate::FILE_PATH.into(),
        ]
    }
}

impl DnsService {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            process: Arc::new(ProcessManager::new(DnsMeta, state_manager).await?),
        })
    }

    pub async fn status(&self) -> Result<ServiceStatus> {
        Ok(ServiceStatus {
            enabled: self.state_manager.config.dns.enabled,
            state: self.process.current_state().await?,
            detail: Default::default(),
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.state_manager.config.dns.enabled {
            UnboundConfigTemplate::write(self.state_manager.config.clone()).await?;
            self.process.clone().spawn().await?;
        } else {
            info!("DNS service is disabled");
        }
        Ok(())
    }
}
