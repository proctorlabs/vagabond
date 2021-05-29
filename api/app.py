from quart import Quart, jsonify, request
import os
import logging
import asyncio
from .services import Unbound, IPTables, Hostapd, Wireguard, Dhcpd, Interfaces
from .templates import Templates
from .config import Config


def create_app(config_object=""):
    app = Quart(__name__)
    app.logger.info('Application starting...')
    log = logging.getLogger('quart.app')
    config = Config("/etc/vagabond.toml")
    templates = Templates(config)

    hostapd = Hostapd(config, templates)
    iptables = IPTables(config, templates)
    interfaces = Interfaces(config, templates)
    wireguard = Wireguard(config, templates)
    dhcpd = Dhcpd(config, templates)
    unbound = Unbound(config, templates)

    async def start_services():
        log.info("Starting services...")
        await asyncio.gather(
            interfaces.start(),
            iptables.start(),
            hostapd.start(),
        )
        await asyncio.gather(
            unbound.start(),
            dhcpd.start(),
            wireguard.start(),
        )
        log.info("Services started!")

    @app.while_serving
    async def lifespan():
        asyncio.create_task(start_services())
        yield
        log.warning("Shutting down!")

    @app.route('/api/status')
    async def status():
        return jsonify({
            'status': 'running',
            'hostapd': await hostapd.status(),
            'iptables': await iptables.status(),
            'interfaces': await interfaces.status(),
            'wireguard': await wireguard.status(),
            'unbound': await unbound.status(),
            'dhcpd': await dhcpd.status(),
        })

    @app.route('/api/ping')
    async def ping():
        return ""

    return app
