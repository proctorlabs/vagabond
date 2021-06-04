from dbus_next.aio import MessageBus
from dbus_next.introspection import Node
from dbus_next import BusType
from abc import ABC, abstractmethod, abstractproperty

import inspect
import logging
import asyncio

from .iwd_objects import IWDAdapter, IWDDevice, IWDKnownNetwork, IWDNetwork, IWDStation


log = logging.getLogger('quart.app')

OBJECT_MANAGER = 'org.freedesktop.DBus.ObjectManager'
IWD_BASE_OBJECT = 'net.connman.iwd'
STATION_OBJECT = f'{IWD_BASE_OBJECT}.Station'
DEVICE_OBJECT = f'{IWD_BASE_OBJECT}.Device'
ADAPTER_OBJECT = f'{IWD_BASE_OBJECT}.Adapter'
NETWORK_OBJECT = f'{IWD_BASE_OBJECT}.Network'
KNOWN_NETWORK_OBJECT = f'{IWD_BASE_OBJECT}.KnownNetwork'


class IWDManager(object):
    def __init__(self):
        self.bus = None
        self.devices = dict()

    async def bus_loop(self):
        while True:
            try:
                self.bus = await MessageBus(bus_type=BusType.SYSTEM).connect()
                self.introspection = await self.bus.introspect(IWD_BASE_OBJECT, '/')
                await self.bus.wait_for_disconnect()
            except Exception as e:
                log.warning("dbus connection error: %s", e)
            finally:
                self.bus = None
                await asyncio.sleep(10)
                if self.bus != None:
                    log.warning("Likely race condition, dbus loop dying...")
                    return

    async def setup(self):
        if self.bus == None:
            asyncio.create_task(self.bus_loop())

    async def get_bus(self):
        while self.bus == None:
            await asyncio.sleep(0.5)
        return self.bus

    async def get_interface_at_path(self, interface: str, path: str):
        bus = await self.get_bus()
        path_introspect = await bus.introspect(IWD_BASE_OBJECT, path)
        new_obj = bus.get_proxy_object(IWD_BASE_OBJECT, path, path_introspect)
        return new_obj.get_interface(interface)

    async def get_objects_of_type(self, full_type: str):
        bus = await self.get_bus()
        results = []
        obj = bus.get_proxy_object(IWD_BASE_OBJECT, '/', Node.default())
        mgr = obj.get_interface(OBJECT_MANAGER)

        for path, interfaces in (await mgr.call_get_managed_objects()).items():
            if full_type not in interfaces:
                continue
            results.append(await self.get_interface_at_path(full_type, path))

        return results

    async def get_stations(self, proxy_only=True):
        proxies = await self.get_objects_of_type(STATION_OBJECT)
        if proxy_only:
            return proxies
        results = []
        for proxy in proxies:
            obj = IWDStation(self, proxy.path)
            await obj.refresh()
            results.append(obj)
        return results

    async def get_devices(self, proxy_only=True):
        proxies = await self.get_objects_of_type(DEVICE_OBJECT)
        if proxy_only:
            return proxies
        results = []
        for proxy in proxies:
            obj = IWDDevice(self, proxy.path)
            await obj.refresh()
            results.append(obj)
        return results

    async def get_adapters(self, proxy_only=True):
        proxies = await self.get_objects_of_type(ADAPTER_OBJECT)
        if proxy_only:
            return proxies
        results = []
        for proxy in proxies:
            obj = IWDAdapter(self, proxy.path)
            await obj.refresh()
            results.append(obj)
        return results

    async def get_networks(self, proxy_only=True):
        proxies = await self.get_objects_of_type(NETWORK_OBJECT)
        if proxy_only:
            return proxies
        results = []
        for proxy in proxies:
            obj = IWDNetwork(self, proxy.path)
            await obj.refresh()
            results.append(obj)
        return results

    async def get_known_networks(self, proxy_only=True):
        proxies = await self.get_objects_of_type(KNOWN_NETWORK_OBJECT)
        if proxy_only:
            return proxies
        results = []
        for proxy in proxies:
            obj = IWDKnownNetwork(self, proxy.path)
            await obj.refresh()
            results.append(obj)
        return results

    async def get_device(self, name: str):
        for device in await self.get_devices():
            device_name = await device.get_name()
            if device_name == name:
                result = IWDDevice(self, device.path)
                await result.refresh()
                return result
        return None
