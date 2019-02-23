FROM debian:stretch

# Install Rust
RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev libsqlite3-dev:arm64 gcc-aarch64-linux-gnu libc-dev-arm64-cross \
        build-essential debhelper dh-systemd && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-01-09 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add aarch64-unknown-linux-gnu

# Install ghr for GitHub Releases: https://github.com/tcnksm/ghr
RUN curl -L -o ghr.tar.gz https://github.com/tcnksm/ghr/releases/download/v0.12.0/ghr_v0.12.0_linux_amd64.tar.gz && \
    tar -xzf ghr.tar.gz && \
    mv ghr_*_linux_amd64/ghr /usr/bin/ghr

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc