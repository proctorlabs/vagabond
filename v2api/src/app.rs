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
    // iwd: IwdManager,
    system: SystemManager,
    http: HttpServer,
    dns: DnsService,
    dhcp: DhcpService,
    hostapd: HostapdService,
}

impl Vagabond {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        let (http, dns, dhcp, hostapd) = try_join!(
            // IwdManager::new(config.clone()),
            HttpServer::new(config.clone()),
            DnsService::new(config.clone()),
            DhcpService::new(config.clone()),
            HostapdService::new(config.clone()),
        )?;
        Ok(Vagabond {
            system: SystemManager::new(&config),
            config,
            // iwd,
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
            // self.iwd.run_test(),
            self.http.spawn(),
            self.dns.spawn(),
            self.dhcp.spawn(),
            self.hostapd.spawn(),
        )?;

        state::running().await;
        signal::ctrl_c().await?;
        info!("Interrupt received, shutting down...");

        bus::broadcast(Event::Shutdown).unwrap_or_default();
        state::shutdown().await;
        while bus::receiver_count() > 0 {
            sleep(Duration::from_millis(100)).await;
        }
        debug!("Shutdown completed cleanly.");
        Ok(())
    }
}
