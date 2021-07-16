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

RUN mkdir -p /dist/etc/iwd

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
FROM ubuntu:focal as vagabond-builder

RUN echo "resolvconf resolvconf/linkify-resolvconf boolean false" | debconf-set-selections && \
    apt-get update && apt-get install --no-install-recommends -y \
    'dbus' 'resolvconf' 'libdbus-1-dev' 'build-essential' 'curl' 'ca-certificates' && \
    mkdir -p /var/run/dbus /toolchain && \
    rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/toolchain \
    PATH=$PATH:/toolchain/bin \
    RUSTUP_HOME=/toolchain

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
RUN chmod -R 777 /toolchain
WORKDIR /app

# To properly cache dependencies
COPY Cargo.toml Cargo.lock /app/
COPY api/Cargo.toml /app/api/Cargo.toml
RUN mkdir -p api/src/ && touch api/src/main.rs && (cargo build --release || true)

# Full build
COPY api/ /app/api/
RUN cargo build --release

# Build final container
FROM ubuntu:focal as service

RUN echo "resolvconf resolvconf/linkify-resolvconf boolean false" | debconf-set-selections && \
    apt-get update && apt-get install --no-install-recommends -y \
    wireguard iptables hostapd iproute2 iputils-ping nmap iw unbound ca-certificates curl udhcpc udhcpd dbus resolvconf && \
    mkdir -p /var/run/dbus && \
    rm -rf /var/lib/apt/lists/*

COPY --from=iwd-builder /dist/ /
COPY --from=ui /ui/dist/ /app/static/
COPY --from=vagabond-builder /app/target/release/vagabond /app/vagabond
COPY api/dbus/vagabond.conf.xml /usr/share/dbus-1/system.d/vagabond.conf
COPY vagabond.toml /etc/vagabond.toml

EXPOSE 80
WORKDIR /app
# HEALTHCHECK --interval=90s CMD [ "curl", "-sL", "http://localhost:80/api/ping" ]
CMD [ "./vagabond" ]
