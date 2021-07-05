mod dbus_iwd;

use super::*;
use anyhow::Result;
use dbus::{
    arg::{PropMap, RefArg, Variant},
    nonblock::{Proxy, SyncConnection},
    Path,
};
use dbus_tokio::connection;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{sync::RwLock, task::spawn_blocking};

config_file! { IWDConfigTemplate("iwd-main.conf.hbs") => "/data/iwd/etc/main.conf" }

#[derive(Clone)]
pub struct IwdManager {
    conn: Arc<RwLock<Arc<SyncConnection>>>,
    config: VagabondConfig,
}

impl std::fmt::Debug for IwdManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IwdManager").finish()
    }
}

impl IwdManager {
    pub async fn new(config: VagabondConfig) -> Result<Self> {
        let (resource, conn) = spawn_blocking(|| connection::new_system_sync()).await??;
        let conn = Arc::new(RwLock::new(conn));
        let conn2 = conn.clone();
        tokio::spawn(async {
            let mut resource = resource;
            let conn2 = conn2;
            while crate::state::status().await != crate::Status::ShuttingDown {
                let err = resource.await;
                warn!("Lost connection to D-Bus: {}", err);
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                info!("Attempting to reconnect to dbus...");
                let (new_resource, conn) =
                    spawn_blocking(|| connection::new_system_sync()).await??;
                let mut new_conn = conn2.write().await;
                *new_conn = conn;
                resource = new_resource;
            }
            Ok::<(), anyhow::Error>(())
        });
        Ok(IwdManager { conn, config })
    }

    async fn get_connection(&self) -> Result<Arc<SyncConnection>> {
        Ok(self.conn.read().await.clone())
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

macro_rules! dbus_object {
    ($( $traitpath:path => $objname:literal : $name:ident
        <properties> { $( $varname:ident : $vartype:ty ),* }
        <methods> { $( $methodname:ident ($($argname:ident : $argtype:ty),*) -> $returntype:ty ),* }
    )* ) => {
        $(
            #[allow(dead_code)]
            #[derive(Debug)]
            pub struct $name<'a> {
                manager: IwdManager,
                path: Path<'a>,
                populated: bool,
                $( pub $varname: Option<$vartype> , )*
            }

            #[allow(dead_code)]
            impl<'a> $name<'a> {
                pub const DBUS_OBJECT_NAME: &'static str = $objname;

                pub fn new<T: Into<Path<'a>>>(manager: IwdManager, path: T) -> Self {
                    Self {
                        manager,
                        path: path.into(),
                        populated: false,
                        $( $varname: None , )*
                    }
                }

                pub async fn fetch_properties(&mut self) -> Result<()> {
                    if !self.populated {
                        #[allow(unused_variables)]
                        let proxy_object = self.manager.get_proxy(self.path.clone()).await?;
                        $(
                            self.$varname = Some(<Proxy<_> as $traitpath>::$varname(&proxy_object).await.unwrap_or_default());
                        )*
                        self.populated = true;
                    }
                    Ok(())
                }

                $(
                    pub async fn $methodname(&self, $( $argname : $argtype ,)*) -> Result<$returntype> {
                        let proxy_object = self.manager.get_proxy(self.path.clone()).await?;
                        Ok(<Proxy<_> as $traitpath>::$methodname(&proxy_object, $( $argname ,)*).await?)
                    }
                )*
            }
        )*
    };
}

dbus_object! {
    dbus_iwd::Network => "net.connman.iwd.Network" : Network
    <properties> {
        name: String,
        connected: bool,
        type_: String,
        device: Path<'static>,
        known_network: Path<'static>
    }
    <methods> {
        connect() -> ()
    }

    dbus_iwd::Station => "net.connman.iwd.Station" : Station
    <properties> {
        state: String,
        scanning: bool,
        connected_network: Path<'static>
    }
    <methods> {
        connect_hidden_network(name: &str) -> (),
        disconnect() -> (),
        get_ordered_networks() -> Vec<(Path<'static>, i16)>,
        get_hidden_access_points() -> Vec<(String, i16, String)>,
        scan() -> (),
        unregister_signal_level_agent(path: Path<'static>) -> (),
        register_signal_level_agent(path: Path<'static>, levels: Vec<i16>) -> ()
    }

    dbus_iwd::Device => "net.connman.iwd.Device" : Device
    <properties> {
        name: String,
        address: String,
        powered: bool,
        mode: String,
        adapter: Path<'static>
    }
    <methods> {
        set_powered(val: bool) -> (),
        set_mode(mode: String) -> ()
    }

    dbus_iwd::P2pDevice => "net.connman.iwd.p2p.Device" : P2pDevice
    <properties> {
        name: String,
        enabled: bool,
        available_connections: u16
    }
    <methods> {
        get_peers() -> Vec<(Path<'static>, i16)>,
        request_discovery() -> (),
        release_discovery() -> (),
        set_enabled(val: bool) -> (),
        set_name(val: String) -> ()
    }

    dbus_iwd::Adapter => "net.connman.iwd.Adapter" : Adapter
    <properties> {
        name: String,
        model: String,
        vendor: String,
        powered: bool,
        supported_modes: Vec<String>
    }
    <methods> {
        set_powered(val: bool) -> ()
    }

    dbus_iwd::P2pServiceManager => "net.connman.iwd.p2p.ServiceManager" : P2pServiceManager
    <properties> {}
    <methods> {
        register_display_service(props: PropMap) -> (),
        unregister_display_service() -> ()
    }

    dbus_iwd::AgentManager => "net.connman.iwd.AgentManager" : AgentManager
    <properties> {}
    <methods> {
        register_agent(path: Path<'a>) -> (),
        unregister_agent(path: Path<'a>) -> ()
    }

    dbus_iwd::OrgFreedesktopDBusObjectManager => "org.freedesktop.DBus.ObjectManager" : ObjectManager
    <properties> {}
    <methods> {
        get_managed_objects() -> HashMap<Path<'static>, HashMap<String, PropMap>>
    }

    dbus_iwd::OrgFreedesktopDBusProperties => "org.freedesktop.DBus.Properties" : Properties
    <properties> {}
    <methods> {
        get(interface_name: &str, property_name: &str) -> Variant<Box<dyn RefArg + 'static>>,
        // set(interface_name: &str, property_name: &str, value: Variant<Box<dyn RefArg>>) -> (),
        get_all(interface_name: &str) -> PropMap
    }

    dbus_iwd::SimpleConfiguration => "net.connman.iwd.SimpleConfiguration" : SimpleConfiguration
    <properties> {}
    <methods> {
        push_button() -> (),
        generate_pin() -> String,
        start_pin(pin: &str) -> (),
        cancel() -> ()
    }
}
