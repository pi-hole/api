FROM debian:stretch

RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev libsqlite3-dev:armhf gcc-arm-linux-gnueabihf libc6-dev-armhf-cross \
        build-essential debhelper dh-systemd && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-05-11 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add armv7-unknown-linux-gnueabihf

# Install ghr for GitHub Releases: https://github.com/tcnksm/ghr
RUN curl -L -o ghr.tar.gz https://github.com/tcnksm/ghr/releases/download/v0.12.0/ghr_v0.12.0_linux_amd64.tar.gz && \
    tar -xzf ghr.tar.gz && \
    mv ghr_*_linux_amd64/ghr /usr/bin/ghr

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc

LABEL version=1
