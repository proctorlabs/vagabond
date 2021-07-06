#!/usr/bin/env bash

docker run --rm -it -u $UID:$UID \
    -v $PWD/ui:/app -w /app -p 8088:8080 \
    $(docker build -q ui/) yarn build --watch
