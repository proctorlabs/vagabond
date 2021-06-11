import struct
import fcntl
import socket
import logging
import re
import yaml

from pathlib import Path
from functools import cached_property
from .base_interface import BaseInterface
from ..process import Process
from ..commands import Command


log = logging.getLogger('quart.app')
STARTING_TABS = re.compile("^(\t)*")


def get_ip_address(ifname):
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        return socket.inet_ntoa(fcntl.ioctl(
            s.fileno(),
            0x8915,  # SIOCGIFADDR
            struct.pack('256s', ifname[:15].encode())
        )[20:24])
    except:
        return None


class WifiInterface(BaseInterface):
    @property
    def interface_type(self):
        return "wifi"

    @property
    def iwd(self):
        return self._ioc.iwd_manager

    @cached_property
    def phy_name(self):
        if self.phy_name_path.exists():
            with open(self.phy_name_path, "r") as f:
                return f.read()
        return None

    @cached_property
    def phy_path(self):
        return self.sysfs_path / "phy80211"

    @cached_property
    def phy_name_path(self):
        return self.phy_path / "name"

    async def start(self):
        await super().start()

    async def wifi_status(self):
        try:
            device = await self.iwd.get_device(self.interface)
            if device:
                station = await device.get_station()
                network = await station.get_connected_network()
                result = dict()
                result['name'] = device.name
                result['address'] = device.address
                result['mode'] = device.mode
                result['state'] = station.state
                result['ip'] = get_ip_address(device.name)
                if network:
                    result['ssid'] = network.name
                    result['rssi'] = network.rssi
                return result
        except Exception as e:
            log.warning("Failed wifi status due to failure!\n%s", e)
        return {'state': 'unknown'}

    async def wifi_scan(self):
        try:
            device = await self.iwd.get_device(self.interface)
            if device:
                station = await device.get_station()
                scan_results = await station.get_networks()
                results = []
                for item in scan_results:
                    results.append({
                        'ssid': item.name,
                        'signal': item.rssi,
                        'security': item.type,
                        'known': item.has_known_network,
                    })
                if len(results) > 0:
                    await self._ioc.socket_handler.broadcast({
                        'type': 'wifi_scan_result',
                        'data': results,
                    })
        except Exception as e:
            log.warning("Failed wifi scan due to failure!\n%s", e)

    async def connect(self, ssid: str, psk: str):
        append_args = []
        if psk:
            append_args = ['--passphrase', psk]
        res = await Command.run_command('iwctl', [
            'station', self.interface, 'connect', ssid, *append_args
        ])
        log.warning(f"{res.output}")

    async def disconnect(self):
        try:
            device = await self.iwd.get_device(self.interface)
            if device:
                station = await device.get_station()
                await station.disconnect()
        except Exception as e:
            log.warning("Failed wifi disconnect due to failure!\n%s", e)
