import logging
import json
import asyncio
from pathlib import Path
from functools import cached_property

from .dhcp_interface import DhcpInterface
from .wifi_interface import WifiInterface
from .unmanaged_interface import UnmanagedInterface
from .static_interface import StaticInterface

from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


log = logging.getLogger('quart.app')
IWD_ETC = Path("/data/iwd/etc")
IWD_CONF = IWD_ETC / "main.conf"


class Interfaces(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self._ioc = ioc
        self._iwd_manager = ioc.iwd_manager
        self._dbus = Process("dbus", [
            "dbus-daemon", "--system", "--nofork", "--nopidfile", "--nosyslog", "--print-address"
        ])
        self._iwd = Process("iwd", ["/usr/libexec/iwd"])

    async def start(self):
        IWD_ETC.mkdir(exist_ok=True, parents=True)
        self.templates.render("iwd-main.conf.j2", IWD_CONF)
        await self._dbus.start()
        await self._iwd.start("-I", ",".join(self.iwd_interface_blacklist))
        await self._iwd_manager.setup()
        for iface in self.all_interfaces:
            await iface.start()

    async def stop(self):
        await self._iwd.stop()
        await self._dbus.stop()

    def _interface_names(self, interfaces):
        results = []
        for iface in interfaces:
            results.append(iface.interface)
        return results

    @cached_property
    def all_interfaces(self):
        results = []
        if self.lan_interface:
            results.append(self.lan_interface)
        if self.wlan_interface:
            results.append(self.wlan_interface)
        results.extend(self.dhcp_interfaces)
        results.extend(self.wifi_interfaces)
        results.extend(self.unmanaged_interfaces)
        return results

    @cached_property
    def all_interface_names(self):
        return self._interface_names(self.all_interfaces)

    @cached_property
    def dhcp_interfaces(self):
        results = []
        for iface in self.config.network.dhcp_interfaces:
            results.append(DhcpInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def dhcp_interface_names(self):
        return self._interface_names(self.dhcp_interfaces)

    @cached_property
    def wifi_interfaces(self):
        results = []
        for iface in self.config.network.wlan_interfaces:
            results.append(WifiInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def wifi_interface_names(self):
        return self._interface_names(self.wifi_interfaces)

    @cached_property
    def wifi_phy_names(self):
        results = []
        for iface in self.wifi_interfaces:
            results.append(iface.phy_name)
        return results

    @cached_property
    def unmanaged_interfaces(self):
        results = []
        for iface in self.config.network.unmanaged_interfaces:
            results.append(UnmanagedInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def unmanaged_interface_names(self):
        return self._interface_names(self.unmanaged_interfaces)

    @cached_property
    def lan_interface(self):
        if self.config.network.lan_enabled:
            return StaticInterface(
                self.config.network.lan_interface,
                self._ioc,
                self.config.network.lan_address,
                self.config.network.lan_subnet_prefixlen,
            )
        return None

    @cached_property
    def lan_interface_names(self):
        if self.lan_interface:
            return self._interface_names([self.lan_interface])
        else:
            return []

    @cached_property
    def wlan_interface(self):
        if self.config.network.wlan_enabled:
            return StaticInterface(
                self.config.network.wlan_interface,
                self._ioc,
                self.config.network.wlan_address,
                self.config.network.wlan_subnet_prefixlen,
            )
        return None

    @cached_property
    def wlan_interface_names(self):
        if self.wlan_interface:
            return self._interface_names([self.wlan_interface])
        else:
            return []

    @cached_property
    def iwd_interface_blacklist(self):
        result = []
        result.extend(self.wlan_interface_names)
        result.extend(self.lan_interface_names)
        result.extend(self.unmanaged_interface_names)
        result.extend(self.dhcp_interface_names)
        return result

    async def get_interfaces(self):
        if_data = json.loads((await Command.run_command('ip', ['-j', 'addr'])).output)
        result = dict()
        for iface in if_data:
            ifname = iface['ifname']
            if ifname in self.config.network.all_interfaces:
                result[ifname] = iface
        return result

    async def status(self):
        return dict({'interfaces': await self.get_interfaces()})

    async def wifi_scan(self):
        for iface in self.wifi_interfaces:
            asyncio.create_task(iface.wifi_scan())

    async def wifi_status(self):
        for iface in self.wifi_interfaces:
            return await iface.wifi_status()

    async def wifi_connect(self, ssid: str, psk: str):
        if ssid:
            for iface in self.wifi_interfaces:
                await iface.connect(ssid, psk)

    async def wifi_disconnect(self):
        for iface in self.wifi_interfaces:
            await iface.disconnect()
