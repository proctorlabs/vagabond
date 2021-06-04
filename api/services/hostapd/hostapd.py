import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


HOSTAPD_DIRECTORY = Path("/data/hostapd")
HOSTAPD_CONFIG = HOSTAPD_DIRECTORY / "hostapd.conf"


class Hostapd(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self._process = Process("hostapd", [
            "hostapd", "/data/hostapd/hostapd.conf"
        ], ioc=ioc)

    async def status(self):
        return self._process.status

    async def start(self):
        if self.config.network.wlan_enabled:
            HOSTAPD_DIRECTORY.mkdir(parents=True, exist_ok=True)
            self.templates.render("hostapd.conf.j2", HOSTAPD_CONFIG)
            await self._process.start()

    async def stop(self):
        await self._process.stop()
