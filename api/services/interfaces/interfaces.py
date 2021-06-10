import logging
import json
import asyncio

from pathlib import Path
from functools import cached_property
from .dhcp_interface import DhcpInterface
from .wifi_interface import WifiInterface
from .unmanaged_interface import UnmanagedInterface
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
        for iface in self.all_interfaces:
            await iface.start()
        await self.enable_local_interfaces()
        await self._iwd.start("-i", ",".join(self.wifi_interface_names))
        await self._iwd_manager.setup()

    async def stop(self):
        await self._iwd.stop()
        await self._dbus.stop()

    @cached_property
    def wifi_interface_names(self):
        results = []
        for iface in self.wifi_interfaces:
            results.append(iface.interface)
        return results

    @cached_property
    def all_interfaces(self):
        results = []
        results.extend(self.dhcp_interfaces)
        results.extend(self.wifi_interfaces)
        results.extend(self.unmanaged_interfaces)
        if self.lan_interface:
            results.append(self.lan_interface)
        if self.wlan_interface:
            results.append(self.wlan_interface)
        return results

    @cached_property
    def dhcp_interfaces(self):
        results = []
        for iface in self.config.network.dhcp_interfaces:
            results.append(DhcpInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def wifi_interfaces(self):
        results = []
        for iface in self.config.network.wlan_interfaces:
            results.append(WifiInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def unmanaged_interfaces(self):
        results = []
        for iface in self.config.network.unmanaged_interfaces:
            results.append(UnmanagedInterface(iface['interface'], self._ioc))
        return results

    @cached_property
    def lan_interface(self):
        if self.config.network.lan_enabled:
            return UnmanagedInterface(self.config.network.lan_interface, self._ioc)
        return None

    @cached_property
    def wlan_interface(self):
        if self.config.network.wlan_enabled:
            return UnmanagedInterface(self.config.network.wlan_interface, self._ioc)
        return None

    async def enable_local_interfaces(self):
        if self.lan_interface:
            await self.lan_interface.set_address(
                self.config.network.lan_address,
                self.config.network.lan_subnet_prefixlen,
            )
        if self.wlan_interface:
            await self.wlan_interface.set_address(
                self.config.network.wlan_address,
                self.config.network.wlan_subnet_prefixlen,
            )

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
