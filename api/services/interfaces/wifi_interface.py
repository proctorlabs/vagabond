import logging
import re
import yaml

from ..process import Process
from ..commands import Command


log = logging.getLogger('quart.app')
STARTING_TABS = re.compile("^(\t)*")


def iw_list_to_dict(items):
    result = dict()
    props = []
    prop_results = []
    for item in items:
        if isinstance(item, str):
            props.append(item)
        else:
            for k, v in item.items():
                result[k] = v

    for prop in props:
        if ": " in prop:
            k, v = prop.split(": ", 1)
            result[k.strip()] = v.strip()
        else:
            prop_results.append(prop)
    if len(result.keys()) == 0:
        return prop_results
    if len(prop_results) > 0:
        result['Properties'] = prop_results
    return result


def iw_parse_item(item):
    item = item.lstrip('\t').lstrip().rstrip(':').strip().lstrip('*').lstrip()
    if ": " in item:
        k, v = item.split(": ", 1)
        return {k: iw_parse_item(v)}
    else:
        return item


def iw_add_value(to, frm):
    if frm == None:
        return
    elif isinstance(to, dict) and isinstance(frm, dict):
        for k, v in frm.items():
            to[k] = v
    elif isinstance(to, dict):
        to['Properties'] = to.get('Properties', [])
        to['Properties'].append(frm)
    else:
        to.append(frm)


def iw_parse_lines(lines, depth=0):
    ''' Generic parse of the output from an `iw` command. '''
    result = []
    while(len(lines) > 0):
        c = STARTING_TABS.search(lines[0]).span()[1]
        if c > depth:
            if len(result) > 0:
                key = result.pop()
                v = None
                if ":\t" in key:
                    k, v = key.split(":\t", 1)
                    key = k
                    v = iw_parse_item(v)
                newval = iw_parse_lines(lines, depth + 1)
                iw_add_value(newval, v)
                result.append({key: newval})
            else:
                result.append(iw_parse_lines(lines, depth + 1))
        elif c < depth:
            return iw_list_to_dict(result)
        else:
            result.append(lines.pop(0).lstrip(
                '\t').rstrip(':').strip().lstrip('*').lstrip())
    return iw_list_to_dict(result)


def iw_result_to_scan_result(iw_result):
    result = []
    for k, v in iw_result.items():
        result.append({
            'bssid': k[4:21],
            'ssid': v.get("SSID", None),
            'signal': float(v.get("signal", "-120 dBm").split(" ")[0]),
            'channel': int(v.get("DS Parameter set", "channel 0")[8:]),
        })
    result.sort(reverse=True, key=lambda elem: elem['signal'])
    return result


class WifiInterface(object):
    def __init__(self, interface: str):
        self._interface = interface
        self._process = Process(f"{interface} dhcp", [
            "udhcpc", "-i", f"{interface}", "-f",
        ])

    @property
    def interface(self):
        return self._interface

    @property
    def dhcp_running(self):
        return self._process.running

    async def start(self):
        pass
        # await self._process.start()

    async def wifi_scan(self):
        lines = (await Command.run_command('iw', ['dev', self.interface, 'scan', "-u"])).output_lines
        iw_result = iw_parse_lines(lines)
        wifidata = iw_result_to_scan_result(iw_result)
        return wifidata

    @staticmethod
    async def get_wlan_devices(self):
        lines = (await Command.run_command('iw', ['dev'])).output_lines
        result = iw_parse_lines(lines)
        return result

    @staticmethod
    async def get_wlan_interfaces():
        lines = (await Command.run_command('iw', ['list'])).output_lines
        result = iw_parse_lines(lines)
        return result
