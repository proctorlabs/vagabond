#!/usr/bin/env bash

docker run --rm -it -u $UID:$UID \
    -v $PWD/ui:/app -w /app \
    $(docker build -q ui/) yarn build --watch
