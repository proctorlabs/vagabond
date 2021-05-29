import logging
import json

from .dhcp_interface import DhcpInterface
from .wifi_interface import WifiInterface
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


log = logging.getLogger('quart.app')


class Interfaces(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates
        self._wans = []
        self._dhcp_wans = []
        self._wifi_wans = []

    async def start(self):
        await self.enable_local_interfaces()
        await self.enable_wan_interfaces()

    async def enable_wan_interfaces(self):
        for iface in self.config.network.dhcp_interfaces:
            dhcp_iface = DhcpInterface(iface['interface'])
            await dhcp_iface.start()
            self._wans.append(dhcp_iface)
            self._dhcp_wans.append(dhcp_iface)

        for iface in self.config.network.wlan_interfaces:
            wifi_iface = WifiInterface(iface['interface'])
            await wifi_iface.wifi_scan()
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
            result[iface['ifname']] = iface
        return result

    async def status(self):
        return dict({'interfaces': await self.get_interfaces()})
