import logging
from pathlib import Path
from ..commands import Command
from ...config import Config
from ...templates import Templates

log = logging.getLogger('quart.app')

SYSCTL_FILE = Path("/data/sysctl.conf")


class IPTables(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates

    async def start(self):
        # Configure sysctls needed for routing, setup iptables
        self.templates.render("sysctl.conf.j2", SYSCTL_FILE)
        await Command.run_command('sysctl', ['-f', SYSCTL_FILE])
        if self.config.network.manage_routes:
            script = self.templates.render_string("iptables.sh.j2")
            ipt_result = await Command.run_script(script)
            log.info("Iptables output: %s", ipt_result.output)

    async def status(self):
        return dict({
            'version': (await Command.run_command('iptables', ['-V'])).output,
            'tables': (await Command.run_command('iptables', ['-S'])).output,
        })
