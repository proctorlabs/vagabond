#!/usr/bin/env bash

ln -sf ./ui/dist ./static
docker run --rm -it -u $UID:$UID \
    -v $PWD:/app -w /app --privileged --net host \
    $(docker build -q v2api/) bash
