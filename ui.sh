#!/usr/bin/env bash

docker run --rm -it -u $UID:$UID --name vagabond_ui_builder \
    -v $PWD/ui:/app -w /app \
    $(docker build -q ui/) yarn build --watch
