import ZODB
import ZODB.FileStorage
from pathlib import Path


DB_PATH = Path("/data/vagabond.dat")


def _create_storage():
    storage = ZODB.FileStorage.FileStorage(DB_PATH)
    db = ZODB.DB(storage)
    connection = db.open()
    return connection.root


STORE = _create_storage()
