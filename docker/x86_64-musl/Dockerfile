FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates libc-dev musl-tools git \
        libsqlite3-dev build-essential debhelper dh-systemd && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-01-09 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add x86_64-unknown-linux-musl

ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup component add rustfmt clippy

# Install ghr for GitHub Releases: https://github.com/tcnksm/ghr
RUN curl -L -o ghr.tar.gz https://github.com/tcnksm/ghr/releases/download/v0.12.0/ghr_v0.12.0_linux_amd64.tar.gz && \
    tar -xzf ghr.tar.gz && \
    mv ghr_*_linux_amd64/ghr /usr/bin/ghr

ENV TARGET_CC=musl-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=musl-gcc \
    CC_x86_64_unknown_linux_musl=musl-gcc
