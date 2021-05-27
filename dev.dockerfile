FROM node:lts-alpine AS ui

RUN npm install -g @vue/cli && \
    yarn global add @vue/cli-service-global

ENTRYPOINT []

FROM ubuntu:focal as service

RUN apt-get update && apt-get install --no-install-recommends -y \
    wireguard iptables hostapd python3-pip iproute2 iputils-ping nmap iw unbound isc-dhcp-server curl

RUN pip install \
    quart \
    jinja2 \
    toml \
    ZODB

HEALTHCHECK --interval=3s --timeout=3s --start-period=10s --retries=3 CMD [ "curl", "-sL", "http://localhost:5000/api/ping" ]
ENTRYPOINT []
