import logging
from ..commands import Command
from ...config import Config
from ...templates import Templates

log = logging.getLogger('quart.app')


class IPTables(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates

    async def start(self):
        # Configure sysctls needed for routing, setup iptables
        result = await Command.run_script('''\
sysctl net/ipv4/ip_forward=1
sysctl net/ipv6/conf/default/forwarding=1
sysctl net/ipv6/conf/all/forwarding=1
sysctl net/ipv4/icmp_echo_ignore_broadcasts=1
sysctl net/ipv4/icmp_ignore_bogus_error_responses=1
sysctl net/ipv4/icmp_echo_ignore_all=0
sysctl net/ipv4/conf/all/log_martians=0
sysctl net/ipv4/conf/default/log_martians=0
''')
        log.info(result.output)

    async def status(self):
        return dict({
            'version': (await Command.run_command('iptables', ['-V'])).output,
            'tables': (await Command.run_command('iptables', ['-S'])).output,
        })
