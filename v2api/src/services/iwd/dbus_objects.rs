use super::*;
use anyhow::Result;
use dbus::{
    arg::{PropMap, RefArg, Variant},
    nonblock::Proxy,
    Path,
};
use std::collections::HashMap;

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
