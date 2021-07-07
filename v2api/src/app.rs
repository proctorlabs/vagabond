use super::*;
use crate::system::SystemManager;
use anyhow::Result;
use bus::Event;
use services::*;
use tokio::time::{sleep, Duration};
use tokio::{signal, try_join};

#[derive(Debug, Clone)]
pub struct Vagabond {
    config: VagabondConfig,
    state: StateManager,
    system: SystemManager,
    http: HttpServer,
    iwd: IwdManager,
    dns: DnsService,
    dhcp: DhcpService,
    hostapd: HostapdService,
}

impl Vagabond {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        let state = StateManager::new(config.clone()).await?;
        let (iwd, http, dns, dhcp, hostapd) = try_join!(
            IwdManager::new(state.clone()),
            HttpServer::new(state.clone()),
            DnsService::new(state.clone()),
            DhcpService::new(state.clone()),
            HostapdService::new(state.clone()),
        )?;
        Ok(Vagabond {
            system: SystemManager::new(&config),
            config,
            state,
            iwd,
            http,
            dns,
            dhcp,
            hostapd,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        self.system.setup_sysctl()?;
        self.system.setup_iptables().await?;
        try_join!(
            self.iwd.spawn(),
            self.http.spawn(),
            self.dns.spawn(),
            self.dhcp.spawn(),
            self.hostapd.spawn(),
        )?;

        self.iwd.run_test().await?;
        self.state.transition(Status::Running).await;
        signal::ctrl_c().await?;
        info!("Interrupt received, shutting down...");

        bus::broadcast(Event::Shutdown).unwrap_or_default();
        self.state.transition(Status::ShuttingDown).await;
        while bus::receiver_count() > 0 {
            sleep(Duration::from_millis(100)).await;
        }
        debug!("Shutdown completed cleanly.");
        Ok(())
    }
}
