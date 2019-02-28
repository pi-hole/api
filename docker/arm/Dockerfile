FROM debian:stretch

RUN dpkg --add-architecture armel && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev libsqlite3-dev:armel gcc-arm-linux-gnueabi libc-dev-armel-cross \
        build-essential debhelper dh-systemd && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-01-09 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add arm-unknown-linux-gnueabi

# Install ghr for GitHub Releases: https://github.com/tcnksm/ghr
RUN curl -L -o ghr.tar.gz https://github.com/tcnksm/ghr/releases/download/v0.12.0/ghr_v0.12.0_linux_amd64.tar.gz && \
    tar -xzf ghr.tar.gz && \
    mv ghr_*_linux_amd64/ghr /usr/bin/ghr

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=arm-linux-gnueabi-gcc \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABI_LINKER=arm-linux-gnueabi-gcc \
    CC_arm_unknown_linux_gnueabi=arm-linux-gnueabi-gcc