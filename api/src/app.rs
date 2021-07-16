use super::*;
use crate::system::SystemManager;
use anyhow::Result;
use services::*;
use std::sync::Arc;
use tokio::{signal, try_join};

#[derive(Debug, Clone)]
pub struct VagabondInner {
    pub config: VagabondConfig,
    pub state: StateManager,
    pub system: SystemManager,
    pub http: HttpServer,
    pub iwd: IwdManager,
    pub dns: DnsService,
    pub dhcp: DhcpServer,
    pub hostapd: HostapdService,
}

#[derive(Debug, Clone, Deref)]
pub struct Vagabond(Arc<VagabondInner>);

impl Vagabond {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        let state = StateManager::new(config.clone()).await?;
        let system = SystemManager::new(state.clone());
        let (iwd, http, dns, dhcp, hostapd) = try_join!(
            IwdManager::new(state.clone(), system.clone()),
            HttpServer::new(state.clone()),
            DnsService::new(state.clone()),
            DhcpServer::new(state.clone()),
            HostapdService::new(state.clone()),
        )?;
        let result = Vagabond(Arc::new(VagabondInner {
            system,
            config,
            state,
            iwd,
            http,
            dns,
            dhcp,
            hostapd,
        }));
        result.state.set_app_instance(result.clone()).await?;
        Ok(result)
    }

    pub async fn start(&self) -> Result<()> {
        self.system.setup_sysctl()?;
        self.system.setup_iptables().await?;
        self.system.setup_interfaces().await?;
        try_join!(
            self.iwd.spawn_dbus(),
            self.iwd.spawn_iwd(),
            self.iwd.spawn(),
            self.http.spawn(),
            self.dns.spawn(),
            self.dhcp.spawn(),
            self.hostapd.spawn(),
        )?;

        self.state.finish_startup().await;
        signal::ctrl_c().await?;
        info!("Interrupt received, shutting down...");
        self.state.shutdown().await?;
        debug!("Shutdown completed cleanly.");
        Ok(())
    }
}
