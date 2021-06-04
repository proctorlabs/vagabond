import logging
from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


UNBOUND_DIRECTORY = Path("/data/unbound")
UNBOUND_CONFIG = UNBOUND_DIRECTORY / "unbound.conf"


class Unbound(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self._process = Process("unbound", [
            "unbound", "-d", "-p", "-c", "/data/unbound/unbound.conf"
        ], ioc=ioc)

    async def status(self):
        return self._process.status

    async def start(self):
        if self.config.dns.enabled:
            UNBOUND_DIRECTORY.mkdir(parents=True, exist_ok=True)
            self.templates.render("unbound.conf.j2", UNBOUND_CONFIG)
            await self._process.start()

    async def stop(self):
        await self._process.stop()
