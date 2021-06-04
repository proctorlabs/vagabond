import logging
from jinja2 import Environment, FileSystemLoader, Template, select_autoescape
from functools import cached_property
from pathlib import Path


log = logging.getLogger('quart.app')
TEMPLATE_DIR = (Path(__file__).parent.absolute()) / "tpl"


class Templates:
    def __init__(self, ioc):
        self._config = ioc.config

    @cached_property
    def context(self):
        return self._config.asdict()

    @cached_property
    def environment(self):
        return Environment(
            loader=FileSystemLoader(TEMPLATE_DIR),
            autoescape=select_autoescape(
                default_for_string=False,
                default=False,
            ),
        )

    def render_string(self, name: str):
        template = self.environment.get_template(name)
        return template.render(self.context)

    def render(self, name: str, dest: Path):
        template = self.environment.get_template(name)
        contents = template.render(self.context)
        with open(dest, "w") as f:
            f.write(contents)
