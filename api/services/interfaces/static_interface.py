import logging
from ..process import Process
from .base_interface import BaseInterface

log = logging.getLogger('quart.app')


class StaticInterface(BaseInterface):
    def __init__(self, interface: str, ioc, address, prefix):
        super().__init__(interface, ioc)
        self._address = address
        self._prefix = prefix

    @property
    def interface_type(self):
        return "static"

    @property
    def static_address(self):
        return self._address

    @property
    def static_prefix(self):
        return self._prefix

    async def start(self):
        await super().start()
        await self.set_address(self.static_address, self.static_prefix)
