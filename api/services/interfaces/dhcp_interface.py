import logging
from ..process import Process
from .base_interface import BaseInterface

log = logging.getLogger('quart.app')


class DhcpInterface(BaseInterface):
    def __init__(self, interface: str, ioc):
        super().__init__(interface, ioc)
        self._process = Process(f"{interface} dhcp", [
            "udhcpc", "-i", f"{interface}", "-f",
        ], ioc=ioc)

    @property
    def interface_type(self):
        return "dhcp"

    @property
    def dhcp_running(self):
        return self._process.running

    async def start(self):
        await super().start()
        await self._process.start()
