# Build connman
FROM ubuntu:focal as connman-builder
RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y libdbus-1-dev build-essential
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y git automake
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y libtool
RUN mkdir /build && cd /build && \
    git config --global advice.detachedHead false && \
    git clone --branch 1.40 --depth 1 git://git.kernel.org/pub/scm/network/connman/connman.git

WORKDIR /build/connman
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y libreadline-dev libglib2.0-0 libglib2.0-dev libxtables-dev libmnl-dev libgnutls28-dev libgnutls30 libnftnl-dev

RUN ./bootstrap && ./configure \
    --prefix=/usr \
    --localstatedir=/var \
    --sysconfdir=/etc \
    --with-firewall=nftables \
    --enable-openconnect=no \
    --enable-openvpn=no \
    --enable-vpnc=no \
    --enable-session-policy-local=no \
    --enable-nmcompat=no \
    --enable-polkit \
    --enable-iwd \
    --disable-wifi \
    --disable-datafiles

RUN make && make install
