FROM debian:stretch

# Install Rust
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc gcc-multilib git \
        libsqlite3-dev:i386 build-essential debhelper dh-systemd && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-05-11 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add i686-unknown-linux-gnu

# Install ghr for GitHub Releases: https://github.com/tcnksm/ghr
RUN curl -L -o ghr.tar.gz https://github.com/tcnksm/ghr/releases/download/v0.12.0/ghr_v0.12.0_linux_amd64.tar.gz && \
    tar -xzf ghr.tar.gz && \
    mv ghr_*_linux_amd64/ghr /usr/bin/ghr

ENV PATH="/root/.cargo/bin:$PATH"

LABEL version=1
