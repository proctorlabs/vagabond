import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


log = logging.getLogger('quart.app')

DHCPD_DIRECTORY = Path("/data/dhcpd")
DHCPD_LEASES = DHCPD_DIRECTORY / "dhcpd.leases"
DHCPD_CONFIG = DHCPD_DIRECTORY / "dhcpd.conf"


class Dhcpd(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates
        self._process = Process("isc-dhcpd-server", [
            "dhcpd", "-cf", "/data/dhcpd/dhcpd.conf", "-lf", "/data/dhcpd/dhcpd.leases", "-f", "-q", "--no-pid"
        ])

    async def status(self):
        return dict({
            'version': (await Command.run_command('dhcpd', ['--version'])).output,
        })

    async def start(self):
        if self.config.dhcp.enabled:
            DHCPD_DIRECTORY.mkdir(parents=True, exist_ok=True)
            DHCPD_LEASES.touch(exist_ok=True)
            self.templates.render("dhcpd.conf.j2", DHCPD_CONFIG)
            await self._process.start()
