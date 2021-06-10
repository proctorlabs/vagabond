from quart import Quart, jsonify, request, websocket
import os
import logging
import asyncio
import json
from .ioc import IOC


# https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts


def create_app(config_object=""):
    app = Quart(__name__)
    app.logger.info('Application starting...')
    log = logging.getLogger('quart.app')
    ioc = IOC()

    async def start_services():
        log.info("Starting services...")
        await asyncio.gather(
            ioc.interfaces.start(),
            ioc.iptables.start(),
            ioc.hostapd.start(),
        )
        await asyncio.gather(
            ioc.unbound.start(),
            ioc.dhcpd.start(),
            ioc.wireguard.start(),
        )
        log.info("Services started!")

    async def stop_services():
        log.info("Stopping services...")
        await asyncio.gather(
            ioc.interfaces.stop(),
            ioc.iptables.stop(),
            ioc.hostapd.stop(),
            ioc.unbound.stop(),
            ioc.dhcpd.stop(),
            ioc.wireguard.stop(),
        )
        log.info("Services stopped!")

    @app.while_serving
    async def lifespan():
        asyncio.create_task(start_services())
        yield
        await stop_services()

    # API Routes
    @app.websocket('/api/sock')
    async def ws():
        log.info("New socket connection")
        try:
            await ioc.socket_handler.handle_connection()
        except asyncio.CancelledError:
            log.info('Client disconnected')
            raise

    @app.route('/api/status')
    async def status():
        return jsonify({
            'hostapd': await ioc.hostapd.status(),
            'wireguard': await ioc.wireguard.status(),
            'unbound': await ioc.unbound.status(),
            'dhcpd': await ioc.dhcpd.status(),
            'iptables': await ioc.iptables.status(),
            'interfaces': await ioc.interfaces.status(),
        })

    @app.route('/api/ping')
    async def ping():
        return ""

    @app.route('/api/<path:path>')
    async def defaultapi(path):
        return jsonify({'error': "Not an API endpoint"}), 404

    # Asset Routes
    @app.route('/js/<path:path>')
    async def js_assets(path):
        return await app.send_static_file(f"js/{path}")

    @app.route('/css/<path:path>')
    async def css_assets(path):
        return await app.send_static_file(f"css/{path}")

    @app.route('/fonts/<path:path>')
    async def font_assets(path):
        return await app.send_static_file(f"fonts/{path}")

    @app.route('/favicon.ico')
    async def favicon():
        return await app.send_static_file("favicon.ico")

    @app.route('/', defaults={'path': 'index.html'})
    @app.route('/<path>')
    async def indexfile(path):
        return await app.send_static_file("index.html")

    return app
