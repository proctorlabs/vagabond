import logging
from pathlib import Path
from ..commands import Command
from ..service import Service
from ...config import Config
from ...templates import Templates


UNBOUND_DIRECTORY = Path("/data/unbound")
UNBOUND_CONFIG = UNBOUND_DIRECTORY / "unbound.conf"


class Unbound(Service):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates

    @property
    def service_name(self):
        return "unbound"

    @property
    def service_command(self):
        return ["unbound", "-d", "-p", "-c", "/data/unbound/unbound.conf"]

    async def status(self):
        return dict({
            'version': (await Command.run_command('unbound', ['-V'])).output_lines[0]
        })

    async def start(self):
        if self.config.dns.enabled:
            UNBOUND_DIRECTORY.mkdir(parents=True, exist_ok=True)
            self.templates.render("unbound.conf.j2", UNBOUND_CONFIG)
            await self.initialize_service()
