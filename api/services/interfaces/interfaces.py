import logging
import json
import re
from ..commands import Command
from ...config import Config
from ...templates import Templates


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


def iw_parse_lines(lines, depth=0):
    ''' Generic parse of the output from an `iw` command. '''
    result = []
    while(len(lines) > 0):
        c = STARTING_TABS.search(lines[0]).span()[1]
        if c > depth:
            if len(result) > 0:
                key = result.pop()
                result.append({key: iw_parse_lines(lines, depth + 1)})
            else:
                result.append(iw_parse_lines(lines, depth + 1))
        elif c < depth:
            return iw_list_to_dict(result)
        else:
            result.append(lines.pop(0).lstrip(
                '\t').rstrip(':').strip().lstrip('*').lstrip())
    return iw_list_to_dict(result)


class Interfaces(object):
    def __init__(self, config: Config, templates: Templates):
        self.config = config
        self.templates = templates

    async def get_interfaces(self):
        if_data = json.loads((await Command.run_command('ip', ['-j', 'addr'])).output)
        result = dict()
        for iface in if_data:
            result[iface['ifname']] = iface
        return result

    async def get_wifi_interfaces(self):
        lines = (await Command.run_command('iw', ['list'])).output_lines
        result = iw_parse_lines(lines)
        return result

    async def get_wifi_devices(self):
        lines = (await Command.run_command('iw', ['dev'])).output_lines
        result = iw_parse_lines(lines)
        return result

    async def wifi_scan(self):
        lines = (await Command.run_command('iw', ['dev', 'wlp1s0', 'scan'])).output_lines
        result = iw_parse_lines(lines)
        return result

    async def status(self):
        return dict({
            'interfaces': await self.get_interfaces(),
            'wireless': await self.get_wifi_interfaces(),
            'networks': await self.get_wifi_devices(),
        })
