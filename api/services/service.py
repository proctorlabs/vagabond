from abc import ABC, abstractmethod
import logging
import asyncio

log = logging.getLogger('quart.app')


class Service(ABC):
    @property
    @abstractmethod
    def service_name(self):
        return "Generic Service"

    @property
    @abstractmethod
    def service_command(self):
        return ["sleep", "5"]

    async def service_output(self, line, stderr=False):
        if stderr:
            log.warning("%s: %s", self.service_name, line)
        else:
            log.info("%s: %s", self.service_name, line)

    async def read_output(self, stream, stderr=False):
        while True:
            output = await stream.readline()
            if output == b"":
                return
            await self.service_output(output.rstrip().decode('utf-8'))

    async def wait_process(self, proc):
        try:
            await proc.wait()
            log.warning(
                "%s ended with status code %s!",
                self.service_name, proc.returncode
            )
            log.warning(
                "%s will attempt restart in 10 seconds...",
                self.service_name
            )
        finally:
            await asyncio.sleep(10)
            await self.initialize_service()

    async def initialize_service(self):
        log.info("Starting service %s", self.service_name)
        proc = await asyncio.create_subprocess_exec(
            *self.service_command,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
        )

        asyncio.create_task(self.read_output(proc.stdout, False))
        asyncio.create_task(self.read_output(proc.stderr, True))
        asyncio.create_task(self.wait_process(proc))
