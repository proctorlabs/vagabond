import logging
from ..process import Process
from .base_interface import BaseInterface

log = logging.getLogger('quart.app')


class UnmanagedInterface(BaseInterface):
    @property
    def interface_type(self):
        return "unmanaged"
