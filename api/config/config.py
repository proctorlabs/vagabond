import toml
import logging
import ipaddress
from functools import cached_property
from .mixin import ConfigMixin


log = logging.getLogger('quart.app')
CONFIG_NOTICE = """\
# === WARNING ===
# This configuration is automatically generated by vagabond
# Manual edits will be overwritten
# To update values here, update the vagabond configuration and reload\
"""


def deep_merge(left, right, path=[]):
    for key in right:
        if key in left:
            if isinstance(left[key], dict) and isinstance(right[key], dict):
                deep_merge(left[key], right[key], path + [str(key)])
            elif type(left[key]) != type(right[key]):
                raise Exception(
                    f"Unexpected key type for '{'.'.join(path)}.{key}'! Expected {type(left[key]).__name__}")
            else:
                left[key] = right[key]
        else:
            raise Exception(f"Unknown key path '{'.'.join(path)}.{key}'!")

    return left


CONFIG_DEFAULTS = {
    'network': {
        'domain': 'vagabond.lan',
        'wan': [],
        'manage_routes': True,
        'lan': {
            'enabled': True,
            'interface': 'eth0',
            'subnet': '192.168.0.0/24',
            'address': '192.168.0.0',
        },
        'wlan': {
            'enabled': True,
            'interface': 'wlan0',
            'subnet': '192.168.0.0/24',
            'address': '192.168.0.0',
            'ssid': 'vagabond',
            'hostapd_config': '',
            'channel': 0,
        },
    },
    'wireguard': {
        'enabled': False,
        'interface': 'wg0',
        'address': '192.168.0.0',
        'private_key': '',
        'peer': []
    },
    'dns': {
        'enabled': True,
        'block_malicious': True,
        'servers': ['1.1.1.1', '1.0.0.1'],
        'extra_config': "",
    },
    'dhcp': {
        'enabled': True,
        'lan': {
            'range': {
                'start': '192.168.0.100',
                'end': '192.168.0.199',
            }
        },
        'wlan': {
            'range': {
                'start': '192.168.0.100',
                'end': '192.168.0.199',
            }
        },
        'extra_config': "",
    }
}


class Config(ConfigMixin):
    def __init__(self, cfg_path: str):
        self._config = deep_merge(CONFIG_DEFAULTS, toml.load(cfg_path))

    def asdict(self):
        return {
            'config_notice': CONFIG_NOTICE,
            'dns': self.dns.asdict(),
            'network': self.network.asdict(),
            'dhcp': self.dhcp.asdict(),
            'wireguard': self.wireguard.asdict(),
        }

    @cached_property
    def dns(self):
        return Dns(self._config)

    @cached_property
    def network(self):
        return Network(self._config)

    @cached_property
    def dhcp(self):
        return Dhcp(self._config)

    @cached_property
    def wireguard(self):
        return Wireguard(self._config)


class Wireguard(ConfigMixin):
    def __init__(self, cfg: dict):
        self._config = cfg

    def asdict(self):
        return {
            'enabled': self.enabled,
            'address': self.address,
            'interface': self.interface,
            'private_key': self.private_key,
            'peers': self.peers,
        }

    @cached_property
    def enabled(self):
        return self.get_path("wireguard.enabled")

    @cached_property
    def interface(self):
        return self.get_path("wireguard.interface")

    @cached_property
    def address(self):
        return self.get_path("wireguard.address")

    @cached_property
    def private_key(self):
        return self.get_path("wireguard.private_key")

    @cached_property
    def peers(self):
        peers = self.get_path("wireguard.peer")
        results = []
        for peer in peers:
            results.append(deep_merge({
                'public_key': '',
                'allowed_ips': '0.0.0.0/0, ::/0',
                'endpoint': '192.168.0.0:51820',
            }, peer))
        return results


class Network(ConfigMixin):
    def __init__(self, cfg: dict):
        self._config=cfg

    def asdict(self):
        return {
            'domain': self.domain,
            'all_interfaces': self.all_interfaces,
            'external_interfaces': self.external_interfaces,
            'dhcp_interfaces': self.dhcp_interfaces,
            'wlan_interfaces': self.wlan_interfaces,
            'manage_routes': self.manage_routes,
            'lan_address': self.lan_address,
            'lan_enabled': self.lan_enabled,
            'lan_interface': self.lan_interface,
            'lan_subnet': f"{self.lan_subnet}",
            'lan_subnet_prefixlen': self.lan_subnet_prefixlen,
            'lan_subnet_address': self.lan_subnet_address,
            'lan_subnet_mask': self.lan_subnet_mask,
            'lan_subnet_broadcast': self.lan_subnet_broadcast,
            'wlan_address': self.wlan_address,
            'wlan_enabled': self.wlan_enabled,
            'wlan_interface': self.wlan_interface,
            'wlan_ssid': self.wlan_ssid,
            'wlan_channel': self.wlan_channel,
            'wlan_hostapd_config': self.wlan_hostapd_config,
            'wlan_subnet': f"{self.wlan_subnet}",
            'wlan_subnet_prefixlen': self.wlan_subnet_prefixlen,
            'wlan_subnet_address': self.wlan_subnet_address,
            'wlan_subnet_mask': self.wlan_subnet_mask,
            'wlan_subnet_broadcast': self.wlan_subnet_broadcast,
        }

    @ cached_property
    def domain(self):
        return self.get_path("network.domain")

    @ cached_property
    def all_interfaces(self):
        return set(self.external_interfaces + [self.lan_interface] + [self.wlan_interface])

    @ cached_property
    def external_interfaces(self):
        result=[]
        cfg=self.get_path("network.wan")
        if cfg:
            for ifinfo in cfg:
                if ifinfo['type'] in ["dhcp", "wlan"]:
                    result.append(ifinfo['interface'])
        return result

    @ cached_property
    def dhcp_interfaces(self):
        result=[]
        cfg=self.get_path("network.wan")
        if cfg:
            for ifinfo in cfg:
                if ifinfo['type'] == "dhcp":
                    result.append(ifinfo)
        return result

    @ cached_property
    def wlan_interfaces(self):
        result=[]
        cfg=self.get_path("network.wan")
        if cfg:
            for ifinfo in cfg:
                if ifinfo['type'] == "wlan":
                    result.append(ifinfo)
        return result

    @ cached_property
    def manage_routes(self):
        return self.get_path("network.manage_routes")

    @ cached_property
    def lan_address(self):
        return self.get_path("network.lan.address")

    @ cached_property
    def lan_enabled(self):
        return self.get_path("network.lan.enabled")

    @ cached_property
    def lan_interface(self):
        return self.get_path("network.lan.interface")

    @ cached_property
    def lan_subnet(self):
        return ipaddress.ip_network(self.get_path("network.lan.subnet"), strict=False)

    @ property
    def lan_subnet_prefixlen(self):
        return self.lan_subnet.prefixlen

    @ property
    def lan_subnet_address(self):
        return self.lan_subnet.network_address

    @ property
    def lan_subnet_mask(self):
        return self.lan_subnet.netmask

    @ property
    def lan_subnet_broadcast(self):
        return self.lan_subnet.broadcast_address

    @ cached_property
    def wlan_address(self):
        return self.get_path("network.wlan.address")

    @ cached_property
    def wlan_enabled(self):
        return self.get_path("network.wlan.enabled")

    @ cached_property
    def wlan_interface(self):
        return self.get_path("network.wlan.interface")

    @ cached_property
    def wlan_ssid(self):
        return self.get_path("network.wlan.ssid")

    @ cached_property
    def wlan_channel(self):
        return self.get_path("network.wlan.channel")

    @ cached_property
    def wlan_hostapd_config(self):
        return self.get_path("network.wlan.hostapd_config")

    @ cached_property
    def wlan_subnet(self):
        return ipaddress.ip_network(self.get_path("network.wlan.subnet"), strict=False)

    @ property
    def wlan_subnet_prefixlen(self):
        return self.wlan_subnet.prefixlen

    @ property
    def wlan_subnet_address(self):
        return self.wlan_subnet.network_address

    @ property
    def wlan_subnet_mask(self):
        return self.wlan_subnet.netmask

    @ property
    def wlan_subnet_broadcast(self):
        return self.wlan_subnet.broadcast_address


class Dhcp(ConfigMixin):
    def __init__(self, cfg: dict):
        self._config=cfg

    def asdict(self):
        return {
            'enabled': self.enabled,
            'lan_range_start': self.lan_range_start,
            'lan_range_end': self.lan_range_end,
            'wlan_range_start': self.wlan_range_start,
            'wlan_range_end': self.wlan_range_end,
            'extra_config': self.extra_config,
        }

    @ cached_property
    def enabled(self):
        return self.get_path("dhcp.enabled")

    @ cached_property
    def lan_range_start(self):
        return self.get_path("dhcp.lan.range.start")

    @ cached_property
    def lan_range_end(self):
        return self.get_path("dhcp.lan.range.end")

    @ cached_property
    def wlan_range_start(self):
        return self.get_path("dhcp.wlan.range.start")

    @ cached_property
    def wlan_range_end(self):
        return self.get_path("dhcp.wlan.range.end")

    @ cached_property
    def extra_config(self):
        return self.get_path("dhcp.extra_config")


class Dns(ConfigMixin):
    def __init__(self, cfg: dict):
        self._config=cfg

    def asdict(self):
        return {
            'enabled': self.enabled,
            'block_malicious': self.block_malicious,
            'servers': self.servers,
            'extra_config': self.extra_config,
        }

    @ cached_property
    def enabled(self):
        return self.get_path("dns.enabled")

    @ cached_property
    def block_malicious(self):
        return self.get_path("dns.block_malicious")

    @ cached_property
    def servers(self):
        return self.get_path("dns.servers")

    @ cached_property
    def extra_config(self):
        return self.get_path("dns.extra_config")
