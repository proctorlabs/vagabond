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
RUN ./bootstrap && \
    ./configure --prefix=/dist/usr \
    --disable-manual-pages \
    --disable-systemd-service \
    --localstatedir=/data/iwd \
    --with-dbus-datadir=/dist/usr/share && \
    make && make install

RUN mkdir -p /dist/etc/iwd

# Build final container
FROM ubuntu:focal as service

RUN echo "resolvconf resolvconf/linkify-resolvconf boolean false" | debconf-set-selections && \
    apt-get update && apt-get install --no-install-recommends -y \
    wireguard iptables hostapd iproute2 iputils-ping nmap iw unbound curl ca-certificates udhcpc udhcpd dbus resolvconf \
    libdbus-1-dev build-essential && \
    mkdir -p /var/run/dbus /toolchain && \
    rm -rf /var/lib/apt/lists/*

ENV CARGO_HOME=/toolchain \
    PATH=$PATH:/toolchain/bin \
    RUSTUP_HOME=/toolchain

RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
RUN chmod -R 777 /toolchain
COPY --from=iwd-builder /dist/ /

COPY dbus/vagabond.conf.xml /usr/share/dbus-1/system.d/vagabond.conf
