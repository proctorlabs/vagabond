mod dbus_iwd;
mod dbus_objects;

use super::*;
use anyhow::Result;
use async_trait::async_trait;
use dbus::{
    nonblock::{Proxy, SyncConnection},
    Path,
};
use dbus_objects::*;
use dbus_tokio::connection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::spawn_blocking};

config_file! { IWDConfigTemplate("iwd-main.conf.hbs") => "/data/iwd/etc/main.conf" }

#[derive(IsVariant)]
enum DbusState {
    Stopped,
    Connected(Arc<SyncConnection>),
    Failed(String),
}

#[derive(Clone)]
pub struct IwdManager {
    dbus_state: Arc<RwLock<DbusState>>,
    dbus_process: Arc<ProcessManager<DbusMeta>>,
    iwd_process: Arc<ProcessManager<IwdMeta>>,
    state_manager: StateManager,
}

#[derive(Debug, Clone)]
pub struct DbusMeta;
impl ProcessService for DbusMeta {
    const SERVICE_NAME: &'static str = "dbus";
    const COMMAND: &'static str = "dbus-daemon";
    const RESTART_TIME: u64 = 30;

    fn get_args(&self) -> &[&str] {
        &[
            "--system",
            "--nofork",
            "--nopidfile",
            "--nosyslog",
            "--print-address",
        ]
    }
}

#[derive(Debug, Clone)]
pub struct IwdMeta;
impl ProcessService for IwdMeta {
    const SERVICE_NAME: &'static str = "iwd";
    const COMMAND: &'static str = "/usr/libexec/iwd";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> &[&str] {
        &[]
    }
}

impl std::fmt::Debug for IwdManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IwdManager").finish()
    }
}

impl IwdManager {
    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(IwdManager {
            dbus_state: Arc::new(RwLock::new(DbusState::Stopped)),
            dbus_process: Arc::new(ProcessManager::new(DbusMeta, state_manager.clone()).await?),
            iwd_process: Arc::new(ProcessManager::new(IwdMeta, state_manager.clone()).await?),
            state_manager,
        })
    }

    async fn get_connection(&self) -> Result<Arc<SyncConnection>> {
        let cur = self.dbus_state.read().await;
        match &*cur {
            DbusState::Stopped => Err(anyhow::anyhow!("dbus connection not available!")),
            DbusState::Failed(e) => Err(anyhow::anyhow!("dbus in failed state {}", e)),
            DbusState::Connected(conn) => Ok(conn.clone()),
        }
    }

    async fn get_proxy<'a, T: Into<Path<'a>>>(
        &'a self,
        path: T,
    ) -> Result<dbus::nonblock::Proxy<'a, Arc<dbus::nonblock::SyncConnection>>> {
        let conn = self.get_connection().await?;
        Ok(Proxy::new(
            "net.connman.iwd",
            path,
            Duration::from_millis(5000),
            conn,
        ))
    }

    pub async fn spawn_dbus(&self) -> Result<()> {
        self.dbus_process.clone().spawn().await?;
        Ok(())
    }

    pub async fn spawn_iwd(&self) -> Result<()> {
        self.iwd_process.clone().spawn().await?;
        Ok(())
    }

    pub async fn run_test(&self) -> Result<()> {
        let object_manager = ObjectManager::new(self.clone(), "/");
        let names = object_manager.get_managed_objects().await?;

        for (path, vals) in names.iter() {
            match (path, vals) {
                (path, _) if vals.contains_key(Network::DBUS_OBJECT_NAME) => {
                    let mut network = Network::new(self.clone(), path);
                    network.fetch_properties().await?;
                    debug!("Found network {:?}", network);
                }
                (path, _) if vals.contains_key(Device::DBUS_OBJECT_NAME) => {
                    let mut device = Device::new(self.clone(), path);
                    device.fetch_properties().await?;
                    debug!("Found device {:?}", device);
                }
                (path, vals) => {
                    warn!("Unknown object with properties {:?} at path {}", vals, path);
                }
            };
        }
        Ok(())
    }
}

#[async_trait]
impl Service for IwdManager {
    fn name(&self) -> &'static str {
        "IWD Monitor"
    }

    fn restart_time(&self) -> u64 {
        10
    }

    async fn state_manager(&self) -> StateManager {
        self.state_manager.clone()
    }

    async fn start(&self) -> Result<()> {
        if !self.dbus_state.read().await.is_connected() {
            info!("Connecting to iwd via dbus...");
            let (resource, conn) = spawn_blocking(|| connection::new_system_sync()).await??;
            *self.dbus_state.write().await = DbusState::Connected(conn.clone());
            let state_manager = self.state_manager.clone();
            let zelf = self.clone();
            // tokio::spawn(async move {
            let mut resource = resource;
            while state_manager.current_status().await != crate::Status::ShuttingDown {
                let err = resource.await;
                warn!("Lost connection to D-Bus: {}", err);
                *zelf.dbus_state.write().await = DbusState::Failed(err.to_string());
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                info!("Attempting to reconnect to dbus...");
                let (new_resource, new_conn) =
                    spawn_blocking(|| connection::new_system_sync()).await??;
                *zelf.dbus_state.write().await = DbusState::Connected(new_conn.clone());
                resource = new_resource;
            }
            //     Ok::<(), anyhow::Error>(())
            // });
        }
        Ok(())
    }
}
