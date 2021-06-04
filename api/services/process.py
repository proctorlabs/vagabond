import logging
import asyncio
import datetime

log = logging.getLogger('quart.app')


class Process(object):
    def __init__(self, name: str, command, *, restart_time=10, ioc=None):
        self._name = name
        self._command = command
        self._restart_time = restart_time
        self._ioc = ioc
        self.lock = asyncio.Lock()
        self.proc = None
        self.start_timestamp = None
        self._stopped = True
        self._logs = []
        self._last_result = None

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
        if self.start_timestamp:
            ts = self.start_timestamp
            now = datetime.datetime.now()
            return (now - ts).seconds
        else:
            return None

    @property
    def running(self):
        return self.proc != None

    @property
    def logs(self):
        return self._logs

    @property
    def last_result(self):
        return self._last_result

    @property
    def status(self):
        return {
            'running': self.running,
            'stopped': self._stopped,
            'uptime': self.uptime,
            'recent_logs': self.logs,
            'last_result': self.last_result,
        }

    async def _read_output(self, stream, stderr=False):
        while True:
            output = await stream.readline()
            if output == b"":
                return
            await self._process_output(output.rstrip().decode('utf-8'))

    async def _process_output(self, line, stderr=False):
        self._logs.append(f"{'ERR' if stderr else 'LOG'}: {line}")
        while len(self._logs) > 100:
            self._logs.pop(0)

        if stderr:
            log.warning("%s: %s", self.name, line)
        else:
            log.info("%s: %s", self.name, line)
        if self._ioc:
            await self._ioc.socket_handler.broadcast({
                'type': 'log',
                'data': {
                    'service': self.name,
                    'error': stderr,
                    'message': line,
                }
            })

    async def _monitor(self, proc):
        try:
            await proc.wait()
            self._last_result = proc.returncode
            log.warning(
                "%s ended with status code %s!",
                self.name, proc.returncode,
            )
        finally:
            async with self.lock:
                self.proc = None
                self.start_timestamp = None
            if not self._stopped:
                log.warning(
                    "%s will attempt restart in %s seconds...",
                    self.name, self.restart_time,
                )
                await asyncio.sleep(self.restart_time)
                asyncio.create_task(self.start())

    async def start(self, *args):
        log.info("Starting service %s", self.name)
        async with self.lock:
            self._stopped = False
            if self.proc != None:
                log.critical(
                    "Attempted to start service %s while it is already running!")
                return
            self.start_timestamp = datetime.datetime.now()
            self.proc = await asyncio.create_subprocess_exec(
                *self.command, *args,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )

            asyncio.create_task(self._read_output(self.proc.stdout, False))
            asyncio.create_task(self._read_output(self.proc.stderr, True))
            asyncio.create_task(self._monitor(self.proc))

    async def stop(self):
        log.info("Stopping service %s", self.name)
        async with self.lock:
            self._stopped = True
            if self.proc != None:
                self.proc.terminate()
                await self.proc.wait()
