import logging
import json
from abc import ABC, abstractproperty
from ..commands import Command

log = logging.getLogger('quart.app')


class BaseInterface(ABC):
    def __init__(self, interface: str, ioc):
        self._interface = interface
        self._ioc = ioc
        self._running = False
        self._ip_addr = dict()

    @property
    def interface(self):
        return self._interface

    @property
    def status(self):
        return {
            "type": self.interface_type,
            "running": self._running,
            "interface": self._ip_addr,
        }

    @property
    def ifindex(self):
        return self._ip_addr.get("ifindex", -1)

    @property
    def ifname(self):
        return self._ip_addr.get("ifname", self.interface)

    @property
    def mtu(self):
        return self._ip_addr.get("mtu", 1500)

    @property
    def mac_address(self):
        return self._ip_addr.get("address", "ff:ff:ff:ff:ff:ff")

    @property
    def if_operational(self):
        return self._ip_addr.get("operstate", "DOWN") == "UP"

    @property
    def ip4_addresses(self):
        result = []
        for addr_info in self._ip_addr.get("addr_info", []):
            if addr_info.get("family", "") == "inet":
                result.append(addr_info.get("local", "127.0.0.1"))
        return result

    @property
    def ip6_addresses(self):
        result = []
        for addr_info in self._ip_addr.get("addr_info", []):
            if addr_info.get("family", "") == "inet6":
                result.append(addr_info.get("local", "::1"))
        return result

    async def update_interface_properties(self):
        self._ip_addr = json.loads((await Command.run_command("ip", [
            "-j", "address", "show", f"{self.interface}",
        ])).output)[0]

    @abstractproperty
    def interface_type(self):
        pass

    async def start(self):
        await Command.run_command("ip", [
            "link", "set", f"{self.interface}", "up",
        ])
        self._running = True
        await self.update_interface_properties()

    async def stop(self):
        self._running = False
        await self.update_interface_properties()

    async def set_address(self, address, prefix):
        result = await Command.run_command(
            'ip', ['addr', 'change', f"{address}/{prefix}", "dev", self.interface])
        if not result.success:
            log.warning(
                "Failed to set if address on %s due to %s", self.interface, result.output)
        return result.success
