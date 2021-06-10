import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


log = logging.getLogger('quart.app')

DHCPD_DIRECTORY = Path("/etc/dhcp")
DHCPD_LIB_DIRECTORY = Path("/var/lib/dhcp")
DHCPD_CONFIG = DHCPD_DIRECTORY / "dhcpd.conf"
DHCPD_LEASES = DHCPD_LIB_DIRECTORY / "dhcpd.leases"


class Dhcpd(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self._process = Process("isc-dhcpd-server", [
            "dhcpd", "-cf", f"{DHCPD_CONFIG}", "-lf", f"{DHCPD_LEASES}", "-f", "--no-pid"
        ], ioc=ioc)

    async def status(self):
        return self._process.status

    async def start(self):
        if self.config.dhcp.enabled:
            DHCPD_DIRECTORY.mkdir(parents=True, exist_ok=True)
            DHCPD_LEASES.touch(exist_ok=True)
            self.templates.render("dhcpd.conf.j2", DHCPD_CONFIG)
            await self._process.start()

    async def stop(self):
        await self._process.stop()
