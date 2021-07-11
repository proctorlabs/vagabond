use super::*;
use anyhow::Result;
use nix::sys::signal::Signal;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct DhcpClientIface(String);
impl ProcessService for DhcpClientIface {
    const SERVICE_NAME: &'static str = "DHCP Client";
    const COMMAND: &'static str = "udhcpc";
    const RESTART_TIME: u64 = 30;

    fn get_args(&self) -> Vec<String> {
        vec!["-i".into(), self.0.clone(), "-f".into()]
    }
}

#[derive(Debug, Clone)]
pub struct DhcpClient {
    process: Arc<ProcessManager<DhcpClientIface>>,
    state_manager: StateManager,
}

impl DhcpClient {
    pub async fn new(state_manager: StateManager, iface: String) -> Result<Self> {
        Ok(Self {
            state_manager: state_manager.clone(),
            process: Arc::new(ProcessManager::new(DhcpClientIface(iface), state_manager).await?),
        })
    }

    pub async fn release(&self) -> Result<()> {
        self.process.signal(Signal::SIGUSR2)
    }

    pub async fn renew(&self) -> Result<()> {
        self.process.signal(Signal::SIGUSR1)
    }

    pub async fn spawn(&self) -> Result<()> {
        self.process.clone().spawn().await
    }
}
