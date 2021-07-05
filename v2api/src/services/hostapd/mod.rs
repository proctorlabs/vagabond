use super::*;
use anyhow::Result;
use std::sync::Arc;

config_file! { HostapdConfigurationTemplate("hostapd.conf.hbs") => "/data/hostapd/hostapd.conf" }

#[derive(Debug, Clone)]
pub struct HostapdService {
    process: Arc<ProcessManager<HostapdMeta>>,
    config: VagabondConfig,
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
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        Ok(Self {
            process: Arc::new(ProcessManager::new(HostapdMeta).await?),
            config,
        })
    }

    pub async fn spawn(&self) -> Result<()> {
        if self.config.dns.enabled {
            // HostapdConfigurationTemplate::write(self.config.clone()).await?;
            self.process.clone().spawn().await?;
        } else {
            info!("DNS service is disabled");
        }
        Ok(())
    }
}
