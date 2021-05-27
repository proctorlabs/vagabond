import logging

log = logging.getLogger('quart.app')


class ConfigMixin(object):
    def get_path(self, path: str):
        val = self._config
        for seg in path.split('.'):
            val = val.get(seg, None)
            if not val:
                return val
        return val
