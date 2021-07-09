use crate::config::VagabondConfig;
use anyhow::{anyhow, Result};
use nix::unistd;
use std::sync::Arc;
use tokio::task::spawn_blocking;

mod ctls;
mod interfaces;
mod routing;

#[derive(Debug)]
pub struct SystemInfo {
    config: VagabondConfig,
    pub is_root: bool,
}

#[derive(Debug, Clone, Deref)]
pub struct SystemManager(Arc<SystemInfo>);

impl SystemManager {
    pub fn new(config: &VagabondConfig) -> Self {
        interfaces::get_interfaces().unwrap();
        let is_root = unistd::geteuid().is_root();
        if !is_root {
            warn!(
                "NOTICE: Some functionality will be disabled as this app is not running as root!"
            );
        }
        SystemManager(Arc::new(SystemInfo {
            config: config.clone(),
            is_root,
        }))
    }

    pub fn setup_sysctl(&self) -> Result<()> {
        if self.is_root {
            ctls::set_sysctls()?;
        } else {
            warn!("Cannot set sysctl when not root!");
        }
        Ok(())
    }

    pub async fn setup_iptables(&self) -> Result<()> {
        if self.is_root {
            let cfg = self.config.clone();
            spawn_blocking(move || routing::setup_routes(&cfg))
                .await?
                .map_err(|e| anyhow!(format!("{}", e)))?;
            Ok(())
        } else {
            warn!("IPTables cannot be setup as a non-root user!");
            Ok(())
        }
    }
}
