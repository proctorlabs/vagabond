# Build IWD
FROM ubuntu:focal as iwd-builder
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y libdbus-1-dev build-essential
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y git automake
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y libtool
RUN mkdir /build && cd /build && \
    git config --global advice.detachedHead false && \
    git clone --branch 1.14 --depth 1 git://git.kernel.org/pub/scm/network/wireless/iwd.git && \
    git clone --branch 0.40 --depth 1 git://git.kernel.org/pub/scm/libs/ell/ell.git

WORKDIR /build/iwd

RUN DEBIAN_FRONTEND=noninteractive apt-get install -y libreadline-dev
RUN sed -i 's/etc\/iwd/data\/iwd\/etc/g' configure.ac && \
    ./bootstrap && \
    ./configure --prefix=/dist/usr \
    --disable-manual-pages \
    --disable-systemd-service \
    --localstatedir=/data/iwd/state \
    --with-dbus-datadir=/dist/usr/share && \
    make && make install

# Build UI
FROM node:lts-alpine AS ui

RUN npm install -g @vue/cli && \
    yarn global add @vue/cli-service-global

COPY ui/ /ui/
WORKDIR /ui
RUN yarn install
# will be in /ui/dist
RUN yarn build

# Build final container
FROM ubuntu:focal as service

RUN echo "resolvconf resolvconf/linkify-resolvconf boolean false" | debconf-set-selections && \
    apt-get update && apt-get install --no-install-recommends -y \
    wireguard iptables hostapd python3-pip iproute2 iputils-ping nmap iw unbound isc-dhcp-server curl udhcpc dbus resolvconf && \
    mkdir -p /var/run/dbus && \
    rm -rf /var/lib/apt/lists/*

RUN pip install \
    quart \
    jinja2 \
    toml \
    ZODB \
    pyyaml \
    dbus-next && \
    rm -rf /root/.cache

COPY --from=iwd-builder /dist/ /
COPY --from=ui /ui/dist/ /vagabond/static/
COPY vagabond.toml /etc/vagabond.toml
COPY api/ /vagabond/

EXPOSE 5000
HEALTHCHECK --interval=90s CMD [ "curl", "-sL", "http://localhost:5000/api/ping" ]
CMD [ "/usr/local/bin/hypercorn", "-w", "1", "-b", "0.0.0.0:5000", "vagabond:create_app()" ]
