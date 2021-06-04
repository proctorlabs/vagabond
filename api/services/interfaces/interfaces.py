import logging
import json
import asyncio

from pathlib import Path
from .dhcp_interface import DhcpInterface
from .wifi_interface import WifiInterface
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
        self._wans = []
        self._dhcp_wans = []
        self._wifi_wans = []
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
        await asyncio.gather(
            self.enable_local_interfaces(),
            self.enable_wan_interfaces(),
        )
        await self._iwd.start("-i", ",".join(self.wifi_interfaces))
        await self._iwd_manager.setup()

    async def stop(self):
        await self._iwd.stop()
        await self._dbus.stop()

    @property
    def wifi_interfaces(self):
        results = []
        for iface in self.config.network.wlan_interfaces:
            results.append(iface['interface'])
        return results

    async def enable_wan_interfaces(self):
        for iface in self.config.network.dhcp_interfaces:
            dhcp_iface = DhcpInterface(iface['interface'], self._ioc)
            await dhcp_iface.start()
            self._wans.append(dhcp_iface)
            self._dhcp_wans.append(dhcp_iface)

        for iface in self.config.network.wlan_interfaces:
            wifi_iface = WifiInterface(iface['interface'], self._ioc)
            await wifi_iface.start()
            self._wans.append(wifi_iface)
            self._wifi_wans.append(wifi_iface)

    async def enable_local_interfaces(self):
        if self.config.network.lan_enabled:
            await self.enable_static_interface(
                self.config.network.lan_interface,
                self.config.network.lan_address,
                self.config.network.lan_subnet_prefixlen,
            )
        if self.config.network.wlan_enabled:
            await self.enable_static_interface(
                self.config.network.wlan_interface,
                self.config.network.wlan_address,
                self.config.network.wlan_subnet_prefixlen,
            )

    async def enable_static_interface(self, interface, address, prefix):
        log.info("Enabling interface %s", interface)
        result = await Command.run_command(
            'ip', ['addr', 'change', f"{address}/{prefix}", "dev", interface])
        if not result.success:
            log.warning(
                "Failed to set if address on %s due to %s", interface, result.output)
        result = await Command.run_command('ip', ['link', 'set', interface, "up"])
        if not result.success:
            log.warning(
                "Failed to enable link on %s due to %s", interface, result.output)

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
        for iface in self._wifi_wans:
            asyncio.create_task(iface.wifi_scan())

    async def wifi_status(self):
        for iface in self._wifi_wans:
            return await iface.wifi_status()

    async def wifi_connect(self, ssid: str, psk: str):
        if ssid:
            for iface in self._wifi_wans:
                await iface.connect(ssid, psk)

    async def wifi_disconnect(self):
        for iface in self._wifi_wans:
            await iface.disconnect()
