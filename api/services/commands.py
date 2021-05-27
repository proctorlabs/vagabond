import subprocess
from functools import cached_property
import asyncio
import logging


log = logging.getLogger('quart.app')


class CommandOutput(object):
    def __init__(self, stdout, process):
        self._stdout = stdout
        self._process = process

    @cached_property
    def output(self):
        return self._stdout.decode().strip()

    @cached_property
    def output_lines(self):
        return self.output.splitlines()

    @cached_property
    def status(self):
        return self._process.returncode

    @property
    def success(self):
        return self.status == 0


class Command(object):
    @staticmethod
    async def run_script(script: str):
        proc = await asyncio.create_subprocess_shell(
            script,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )
        await proc.wait()
        result = await proc.communicate()
        return CommandOutput(result[0], proc)

    @staticmethod
    async def run_command(command: str, args):
        proc = await asyncio.create_subprocess_exec(
            command, *args,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.STDOUT,
        )
        await proc.wait()
        result = await proc.communicate()
        return CommandOutput(result[0], proc)
