import logging
import asyncio
import json

from quart import websocket

log = logging.getLogger('quart.app')


class SocketHandler(object):
    def __init__(self, ioc):
        self.interfaces = ioc.interfaces
        self.dhcpd = ioc.dhcpd
        self.hostapd = ioc.hostapd
        self.iptables = ioc.iptables
        self.wireguard = ioc.wireguard
        self.unbound = ioc.unbound
        self.ws_queues = set()

    async def handle_connection(self):
        await asyncio.gather(
            self.sender_task(),
            self.receiver_task(),
        )

    async def sender_task(self):
        queue = asyncio.Queue()
        try:
            self.ws_queues.add(queue)
            while True:
                data = await queue.get()
                await websocket.send(json.dumps(data))
        finally:
            self.ws_queues.remove(queue)

    async def receiver_task(self):
        while True:
            try:
                data = await websocket.receive()
                data = json.loads(data)
                result = await self.process_message(data)
                if result:
                    await websocket.send(json.dumps(result))
            except Exception as e:
                log.critical(
                    "Unexpected error while processing websocket message:\n%s", e)

    async def broadcast(self, msg):
        for q in self.ws_queues:
            await q.put(msg)

    async def process_message(self, data: dict):
        typ = data['type']
        method_name = f"{typ}_handler"
        method = getattr(self, method_name, None)
        if method == None:
            log.warning("No handler found for message type %s", typ)
            return None
        else:
            return await method(data)

    async def list_interfaces_handler(self, data: dict):
        ifaces = await self.interfaces.get_interfaces()
        return {
            'type': 'interfaces',
            'data': ifaces,
        }

    async def get_status_handler(self, data: dict):
        return {
            'type': 'status',
            'data': {
                'hostapd': await self.hostapd.status(),
                'iptables': await self.iptables.status(),
                'interfaces': await self.interfaces.status(),
                'wireguard': await self.wireguard.status(),
                'unbound': await self.unbound.status(),
                'dhcpd': await self.dhcpd.status(),
            },
        }

    async def wifi_scan_handler(self, data: dict):
        await self.interfaces.wifi_scan()

    async def wifi_status_handler(self, data: dict):
        return {
            'type': 'wifi_status',
            'data': await self.interfaces.wifi_status(),
        }

    async def wifi_connect_handler(self, data: dict):
        log.info('Got wifi connect command')
        await self.interfaces.wifi_connect(
            data.get('data', dict()).get('ssid', None),
            data.get('data', dict()).get('psk', None),
        )

    async def wifi_disconnect_handler(self, data: dict):
        log.info('Got wifi disconnect command')
        await self.interfaces.wifi_disconnect()
