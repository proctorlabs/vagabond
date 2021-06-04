from dbus_next.aio import MessageBus
from dbus_next import BusType
from abc import ABC, abstractmethod, abstractproperty

import inspect
import logging
import asyncio

log = logging.getLogger('quart.app')


IWD_BASE_OBJECT = 'net.connman.iwd'
STATION_OBJECT = f'{IWD_BASE_OBJECT}.Station'
DEVICE_OBJECT = f'{IWD_BASE_OBJECT}.Device'
ADAPTER_OBJECT = f'{IWD_BASE_OBJECT}.Adapter'
NETWORK_OBJECT = f'{IWD_BASE_OBJECT}.Network'
KNOWN_NETWORK_OBJECT = f'{IWD_BASE_OBJECT}.KnownNetwork'


class IWDBaseObject(ABC):
    def __init__(self, ctrl, dbus_path: str):
        self._ctrl = ctrl
        self._dbus_path = dbus_path
        self._dbus_object = None

    def __getattr__(self, attr):
        if attr in self.dbus_properties:
            return getattr(self, f"_{attr}", None)

    async def refresh(self):
        self._dbus_object = await self.ctrl.get_interface_at_path(self.dbus_class, self.dbus_path)
        for prop in self.dbus_properties:
            getter = getattr(self._dbus_object, f"get_{prop}", None)
            if getter:
                try:
                    val = await getter()
                    setattr(self, f"_{prop}", val)
                except:
                    setattr(self, f"_{prop}", None)
            else:
                setattr(self, f"_{prop}", None)

    @abstractproperty
    def dbus_properties(self):
        return []

    @abstractproperty
    def dbus_class(self):
        return IWD_BASE_OBJECT

    @property
    def dbus_object(self):
        return self._dbus_object

    @property
    def dbus_path(self):
        return self._dbus_path

    @property
    def ctrl(self):
        return self._ctrl

    @property
    def properties_dict(self):
        result = dict({
            'dbus_class': self.dbus_class,
            'dbus_path': self.dbus_path,
        })
        for prop in self.dbus_properties:
            result[prop] = getattr(self, prop, None)
        return result

    def print_properties(self):
        log.info("%s: %s", self.dbus_class, self.properties_dict)


class IWDDevice(IWDBaseObject):
    @property
    def dbus_properties(self):
        return ["name", "address", "powered", "mode", "adapter"]

    @property
    def dbus_class(self):
        return DEVICE_OBJECT

    async def get_station(self):
        result = IWDStation(self.ctrl, self.dbus_path)
        await result.refresh()
        return result

    async def get_adapter(self):
        result = IWDAdapter(self.ctrl, self.adapter)
        await result.refresh()
        return result


class IWDAdapter(IWDBaseObject):
    @property
    def dbus_properties(self):
        return ["name", "powered", "model", "vendor", "supported_modes"]

    @property
    def dbus_class(self):
        return ADAPTER_OBJECT


class IWDStation(IWDBaseObject):
    @property
    def dbus_properties(self):
        return ["scanning", "state", "connected_network"]

    @property
    def dbus_class(self):
        return STATION_OBJECT

    async def disconnect(self):
        await self.dbus_object.call_disconnect()

    async def scan(self):
        await self.dbus_object.call_scan()

    async def get_connected_network(self):
        if not self.connected_network:
            return None
        network = IWDNetwork(self.ctrl, self.connected_network)
        await network.refresh()
        return network

    async def get_networks(self):
        result = []
        network_paths = await self.dbus_object.call_get_ordered_networks()
        for path, rssi in network_paths:
            network = IWDNetwork(self.ctrl, path)
            await network.refresh()
            network.rssi = rssi
            result.append(network)
        return result


class IWDNetwork(IWDBaseObject):
    @property
    def dbus_properties(self):
        return ["name", "connected", "type", "known_network"]

    @property
    def dbus_class(self):
        return NETWORK_OBJECT

    @property
    def has_known_network(self):
        return self.known_network != None

    @property
    def rssi(self):
        return getattr(self, "_rssi", None)

    @rssi.setter
    def rssi(self, val):
        self._rssi = float(val) / 100

    async def connect(self):
        await self.dbus_object.call_connect()

    async def get_known_network(self):
        if not self.has_known_network:
            return None
        network = IWDKnownNetwork(self.ctrl, self.known_network)
        await network.refresh()
        return network


class IWDKnownNetwork(IWDBaseObject):
    @property
    def dbus_properties(self):
        return ["name", "type", "last_connected_time", "hidden", "auto_connect"]

    @property
    def dbus_class(self):
        return KNOWN_NETWORK_OBJECT

    async def forget(self):
        await self.dbus_object.call_forget()
