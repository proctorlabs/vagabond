import logging
import asyncio
import datetime

log = logging.getLogger('quart.app')


class Process(object):
    def __init__(self, name: str, command, *, restart_time=10):
        self._name = name
        self._command = command
        self._restart_time = restart_time
        self.lock = asyncio.Lock()
        self.proc = None
        self.start_timestamp = None

    @property
    def name(self) -> str:
        return self._name

    @property
    def command(self):
        return self._command

    @property
    def restart_time(self) -> int:
        return self._restart_time

    @property
    def uptime(self) -> int:
        ts = self.start_timestamp
        now = datetime.datetime.now()
        return (now - ts).seconds

    @property
    def running(self):
        return self.proc != None

    async def _read_output(self, stream, stderr=False):
        while True:
            output = await stream.readline()
            if output == b"":
                return
            await self._process_output(output.rstrip().decode('utf-8'))

    async def _process_output(self, line, stderr=False):
        if stderr:
            log.warning("%s: %s", self.name, line)
        else:
            log.info("%s: %s", self.name, line)

    async def _monitor(self, proc):
        try:
            await proc.wait()
            log.warning(
                "%s ended with status code %s!",
                self.name, proc.returncode,
            )
            log.warning(
                "%s will attempt restart in %s seconds...",
                self.name, self.restart_time,
            )
        finally:
            async with self.lock:
                self.proc = None
                self.start_timestamp = None
            await asyncio.sleep(self.restart_time)
            asyncio.create_task(self.start())

    async def start(self):
        log.info("Starting service %s", self.name)
        async with self.lock:
            if self.proc != None:
                log.critical(
                    "Attempted to start service %s while it is already running!")
                return
            self.start_timestamp = datetime.datetime.now()
            self.proc = await asyncio.create_subprocess_exec(
                *self.command,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )

            asyncio.create_task(self._read_output(self.proc.stdout, False))
            asyncio.create_task(self._read_output(self.proc.stderr, True))
            asyncio.create_task(self._monitor(self.proc))
