import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


HOSTAPD_DIRECTORY = Path("/data/hostapd")
HOSTAPD_CONFIG = HOSTAPD_DIRECTORY / "hostapd.conf"


class Hostapd(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates
        self._process = Process("hostapd", [
            "hostapd", "/data/hostapd/hostapd.conf"
        ])

    async def status(self):
        return dict({
            'version': (await Command.run_command('hostapd', ['-v'])).output_lines[0]
        })

    async def start(self):
        if self.config.network.wlan_enabled:
            HOSTAPD_DIRECTORY.mkdir(parents=True, exist_ok=True)
            self.templates.render("hostapd.conf.j2", HOSTAPD_CONFIG)
            await self._process.start()
