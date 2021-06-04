from functools import cached_property
from .services import Unbound, IPTables, Hostapd, Wireguard, Dhcpd, Interfaces, IWDManager
from .templates import Templates
from .config import Config
from .socket_handler import SocketHandler


class IOC(object):
    @cached_property
    def config(self):
        return Config("/etc/vagabond.toml")

    @cached_property
    def templates(self):
        return Templates(self)

    @cached_property
    def hostapd(self):
        return Hostapd(self)

    @cached_property
    def iptables(self):
        return IPTables(self)

    @cached_property
    def interfaces(self):
        return Interfaces(self)

    @cached_property
    def wireguard(self):
        return Wireguard(self)

    @cached_property
    def dhcpd(self):
        return Dhcpd(self)

    @cached_property
    def unbound(self):
        return Unbound(self)

    @cached_property
    def socket_handler(self):
        return SocketHandler(self)

    @cached_property
    def iwd_manager(self):
        return IWDManager()
