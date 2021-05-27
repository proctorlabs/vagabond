import logging
from pathlib import Path
from ..commands import Command
from ..service import Service
from ...config import Config
from ...templates import Templates


class Wireguard(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates

    async def start(self):
        pass

    async def status(self):
        return dict({
            'status': (await Command.run_command('wg', ['show'])).output_lines,
        })
