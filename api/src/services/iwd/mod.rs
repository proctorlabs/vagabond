mod dbus_iwd;
mod dbus_objects;

use super::*;
use crate::data::*;
use crate::util::run_command;
use anyhow::Result;
use async_trait::async_trait;
use dbus::channel::MatchingReceiver;
use dbus::{
    arg::RefArg,
    message::MatchRule,
    nonblock::{Proxy, SyncConnection},
    Path,
};
use dbus_objects::*;
use dbus_tokio::connection;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::spawn_blocking};

use dbus_crossroads::{Context, Crossroads};

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
    const SERVICE_NAME: &'static str = "Dbus Daemon";
    const COMMAND: &'static str = "dbus-daemon";
    const RESTART_TIME: u64 = 30;

    fn get_args(&self) -> Vec<String> {
        vec![
            "--system".into(),
            "--nofork".into(),
            "--nopidfile".into(),
            "--nosyslog".into(),
            "--print-address".into(),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct IwdMeta(Vec<String>);
impl ProcessService for IwdMeta {
    const SERVICE_NAME: &'static str = "Wireless Daemon";
    const COMMAND: &'static str = "/usr/libexec/iwd";
    const RESTART_TIME: u64 = 8;

    fn get_args(&self) -> Vec<String> {
        vec!["-I".into(), self.0.join(",")]
    }
}

impl std::fmt::Debug for IwdManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IwdManager").finish()
    }
}

impl IwdManager {
    const BASE_PATH: &'static str = "/";

    pub async fn new(state_manager: StateManager) -> Result<Self> {
        Ok(IwdManager {
            dbus_state: Arc::new(RwLock::new(DbusState::Stopped)),
            dbus_process: Arc::new(ProcessManager::new(DbusMeta, state_manager.clone()).await?),
            iwd_process: Arc::new(
                ProcessManager::new(
                    IwdMeta(state_manager.config.network.wifi_wan_interfaces()),
                    state_manager.clone(),
                )
                .await?,
            ),
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

    #[allow(unused)]
    async fn get_all<'a, T: DbusObject<'a>>(&'a self) -> Result<Vec<T>> {
        let mut result = vec![];
        let object_manager = ObjectManager::new(self.clone(), "/");
        let names = object_manager.get_managed_objects().await?;
        for (path, vals) in names.iter() {
            match (path, vals) {
                (path, _) if vals.contains_key(T::DBUS_OBJECT_NAME) => {
                    let mut dbobject = T::new(self.clone(), path.clone());
                    dbobject.fetch_properties().await?;
                    result.push(dbobject);
                }
                _ => {}
            };
        }
        return Ok(result);
    }

    async fn get_first<'a, T: DbusObject<'a>>(&'a self) -> Result<Option<T>> {
        let object_manager = ObjectManager::new(self.clone(), "/");
        let names = object_manager.get_managed_objects().await?;
        for (path, vals) in names.iter() {
            match (path, vals) {
                (path, _) if vals.contains_key(T::DBUS_OBJECT_NAME) => {
                    let mut dbobject = T::new(self.clone(), path.clone());
                    dbobject.fetch_properties().await?;
                    return Ok(Some(dbobject));
                }
                _ => {}
            };
        }
        return Ok(None);
    }

    async fn get_all_networks<'a>(&'a self) -> Result<Vec<Network<'a>>> {
        let mut result = vec![];
        let object_manager = ObjectManager::new(self.clone(), "/");
        let names = object_manager.get_managed_objects().await?;
        for (path, vals) in names.iter() {
            match (path, vals) {
                (path, _) if vals.contains_key(Network::DBUS_OBJECT_NAME) => {
                    let mut network = Network::new(self.clone(), path.clone());
                    network.fetch_properties().await?;
                    result.push(network);
                }
                _ => {}
            };
        }
        Ok(result)
    }

    #[allow(unused)]
    async fn find_network<'a>(&'a self, ssid: &str) -> Result<Network<'a>> {
        let ssid = Some(String::from(ssid));
        for network in self.get_all_networks().await? {
            if network.name == ssid {
                return Ok(network);
            }
        }
        Err(anyhow::anyhow!("Network SSID not found!"))
    }

    pub async fn disconnect(&self) -> Result<()> {
        let station: Option<Station> = self.get_first().await?;
        if let Some(station) = station {
            station.disconnect().await?;
        }
        Ok(())
    }

    pub async fn connect(&self, params: &ConnectionParameters) -> Result<()> {
        let device: Option<Device> = self.get_first().await?;
        if device.is_none() {
            return Err(anyhow::anyhow!("No wireless devices found!"));
        }
        let device = device.unwrap();
        let name = device.name.unwrap();
        match params {
            ConnectionParameters::Ssid { ssid } => {
                run_command("iwctl", ["station", &name, "connect", &ssid, "--dont-ask"]).await?;
            }
            ConnectionParameters::PresharedKey { ssid, psk } => {
                run_command(
                    "iwctl",
                    [
                        "station",
                        &name,
                        "connect",
                        &ssid,
                        "--passphrase",
                        &psk,
                        "--dont-ask",
                    ],
                )
                .await?;
            }
        };
        Ok(())
    }

    pub async fn get_wifi_device(&self) -> Result<WifiDevice> {
        //Find device
        let station: Option<Station> = self.get_first().await?;
        if station.is_none() {
            return Err(anyhow::anyhow!("No wifi stations available!"));
        }

        //Gather device details
        let mut station = station.unwrap();
        let mut device = station.device().await?;
        let mut adapter = device.adapter().await?;
        station.fetch_properties().await?;
        device.fetch_properties().await?;
        adapter.fetch_properties().await?;

        //Collect modes
        let supported_modes: Vec<WifiMode> = adapter
            .supported_modes
            .unwrap_or_default()
            .iter()
            .map(|s| s.as_str().into())
            .collect();
        let state = station.state.as_ref().map(|s| s.as_str()).unwrap().into();
        let mut connected_network = None;
        if state == WifiState::Connected {
            let mut con_net = station.connected_network().await?;
            con_net.fetch_properties().await?;
            connected_network = Some(WifiNetwork {
                ssid: con_net.name.unwrap_or_default(),
                security: con_net.type_.unwrap_or_default().into(),
                signal: -50,
                known: true,
                interface: device.name.clone(),
            });
        }

        Ok(WifiDevice {
            name: device.name.unwrap_or_else(|| "unknown".into()),
            phy: adapter.name.unwrap_or_else(|| "unknown".into()),
            address: device.address.unwrap_or_else(|| "00:00:00:00:00:00".into()),
            powered: device.powered.unwrap_or(true),
            scanning: station.scanning.unwrap_or(false),
            mode: device.mode.unwrap_or_default().as_str().into(),
            model: adapter
                .model
                .map(|f| {
                    if f.as_str() == "" {
                        "unknown".into()
                    } else {
                        f
                    }
                })
                .unwrap_or_else(|| "unknown".into()),
            vendor: adapter
                .vendor
                .map(|f| {
                    if f.as_str() == "" {
                        "unknown".into()
                    } else {
                        f
                    }
                })
                .unwrap_or_else(|| "unknown".into()),
            connected_network,
            state,
            supported_modes,
        })
    }

    pub async fn get_wifi_networks(&self) -> Result<Vec<WifiNetwork>> {
        let mut result = vec![];
        let station: Option<Station> = self.get_first().await?;
        if station.is_none() {
            return Ok(result);
        }
        let station = station.unwrap();
        for (network, rssi) in station.get_ordered_networks().await? {
            let mut network = Network::new(self.clone(), network);
            network.fetch_properties().await?;
            let kn = network.known_network.unwrap_or_default();
            let kn_str = kn.as_str().unwrap_or_default().trim();
            result.push(WifiNetwork {
                ssid: network.name.unwrap_or_default(),
                security: network.type_.unwrap_or_default().into(),
                signal: rssi / 100,
                known: kn_str != Self::BASE_PATH,
                interface: None,
            });
        }
        Ok(result)
    }

    const DBUS_NAME: &'static str = "com.vagabond.manager";
    const SIGNAL_AGENT: &'static str = "net.connman.iwd.SignalLevelAgent";

    async fn setup_dbus_receiver(&self, conn: &Arc<SyncConnection>) -> Result<()> {
        debug!("Setting up crossroads");
        let mut cr = Crossroads::new();
        cr.set_async_support(Some((
            conn.clone(),
            Box::new(|fut| {
                tokio::spawn(fut);
            }),
        )));

        debug!("Creating IWD signal agent interface");
        // dbus-send --system --print-reply --type=method_call --dest=com.vagabond.manager /iwd/agent com.vagabond.manager.Changed string:"test"
        let signal_agent = cr.register(Self::SIGNAL_AGENT, |b| {
            b.method(
                "Changed",
                ("path", "level"),
                (),
                move |_: &mut Context, _: &mut StateManager, (path, level): (Path, u8)| {
                    warn!("Signal update: {} -> {:?}", path, level);
                    Ok(())
                },
            );
            b.method(
                "Release",
                ("path",),
                (),
                move |_: &mut Context, _: &mut StateManager, (path,): (Path,)| {
                    warn!("Released: {}", path);
                    Ok(())
                },
            );
        });
        cr.insert("/iwd/agent", &[signal_agent], self.state_manager.clone());

        debug!("Setting up receiver");
        conn.start_receive(
            MatchRule::new_method_call(),
            Box::new(move |msg, conn| {
                info!("Received a call to: {:?}", msg.member());
                cr.handle_message(msg, conn).unwrap();
                true
            }),
        );

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
            self.setup_dbus_receiver(&conn).await?;
            let zelf = self.clone();
            let zelf2 = self.clone();
            let conn2 = conn.clone();
            let res = tokio::try_join!(
                async move {
                    *zelf.dbus_state.write().await = DbusState::Connected(conn.clone());
                    let err = resource.await;
                    *zelf.dbus_state.write().await = DbusState::Failed(err.to_string());
                    warn!("Lost connection to D-Bus: {}", err);
                    Ok::<(), anyhow::Error>(())
                },
                async {
                    info!("Requesting name");
                    conn2
                        .request_name(Self::DBUS_NAME, false, true, false)
                        .await?;
                    let station = zelf2.get_first::<Station>().await?;
                    if let Some(station) = station {
                        info!("Registering signal level agent");
                        station
                            .register_signal_level_agent(
                                Path::new("/iwd/agent").unwrap(),
                                vec![-50, -55, -60, -65, -70, -75, -80, -85, -90],
                            )
                            .await?;
                        info!("Registered!");
                    }
                    Ok(())
                }
            );
            *self.dbus_state.write().await = DbusState::Stopped;
            res?;
        }
        Ok(())
    }
}
