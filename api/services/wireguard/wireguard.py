import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates

WIREGUARD_DIRECTORY = Path("/data/wireguard")


class Wireguard(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self.wgconf = WIREGUARD_DIRECTORY / \
            f"{self.config.wireguard.interface}.conf"

    async def start(self):
        if self.config.wireguard.enabled:
            WIREGUARD_DIRECTORY.mkdir(parents=True, exist_ok=True)
            self.templates.render("wireguard.conf.j2", self.wgconf)
            await Command.run_command("wg-quick", [
                "wg-quick", "up", f"{self.wgconf}",
            ])

    async def stop(self):
        if self.config.wireguard.enabled:
            await Command.run_command("wg-quick", [
                "wg-quick", "down", f"{self.wgconf}",
            ])

    async def status(self):
        return dict({
            'status': self._process.status,
        })
