import logging
import aiohttp
import asyncio
import re

from pathlib import Path
from ..commands import Command
from ..process import Process
from ...config import Config
from ...templates import Templates


log = logging.getLogger('quart.app')

UNBOUND_DIRECTORY = Path("/etc/unbound")
UNBOUND_CONFIG = UNBOUND_DIRECTORY / "unbound.conf"
UNBOUND_BLOCKLIST = UNBOUND_DIRECTORY / "unbound-blocklist.conf"

HOST_MATCHER = re.compile("^0\.0\.0\.0 (\S+)")


class Unbound(object):
    def __init__(self, ioc):
        self.config = ioc.config
        self.templates = ioc.templates
        self._process = Process("unbound", [
            "unbound", "-d", "-p", "-c", f"{UNBOUND_CONFIG}"
        ], ioc=ioc)

    async def status(self):
        return self._process.status

    async def start(self):
        if self.config.dns.enabled:
            UNBOUND_DIRECTORY.mkdir(parents=True, exist_ok=True)
            UNBOUND_BLOCKLIST.touch(exist_ok=True)
            self.templates.render("unbound.conf.j2", UNBOUND_CONFIG)
            await self.update_blocklist()
            await self._process.start()

    async def stop(self):
        await self._process.stop()

    async def update_blocklist(self):
        blocked_hostnames = ""
        async with aiohttp.ClientSession() as session:
            async with session.get('https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts') as r:
                line = await r.content.readline()
                while line:
                    line = await r.content.readline()
                    match = HOST_MATCHER.match(line.decode('utf-8').strip())
                    if match:
                        blocked_hostnames += f"local-zone: \"{match.group(1)}\" refuse\n"
        with open(UNBOUND_BLOCKLIST, "w") as f:
            f.write(blocked_hostnames)
