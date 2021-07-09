#!/usr/bin/env bash

ln -sf ./ui/dist ./static
docker run --rm -it \
    -v $PWD/.tmp/registry:/toolchain/registry \
    -v $PWD/.tmp/downloads:/toolchain/downloads \
    -v $PWD/.tmp/tmp:/toolchain/tmp \
    -v $PWD/.tmp/update-hashes:/toolchain/update-hashes \
    -v $PWD:/app \
    -v $PWD/.tmp/target:/app/target \
    -w /app --privileged --net host \
    $(docker build -q v2api/) cargo run -- -c dev.toml -l debug
