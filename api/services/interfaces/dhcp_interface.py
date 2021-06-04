import logging
from ..process import Process

log = logging.getLogger('quart.app')


class DhcpInterface(object):
    def __init__(self, interface: str, ioc):
        self._interface = interface
        self._process = Process(f"{interface} dhcp", [
            "udhcpc", "-i", f"{interface}", "-f",
        ], ioc=ioc)

    @property
    def interface(self):
        return self._interface

    @property
    def dhcp_running(self):
        return self._process.running

    async def start(self):
        await self._process.start()
