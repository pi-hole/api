FROM debian:stretch

# Install Rust
RUN dpkg --add-architecture i386 && \
    apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc gcc-multilib git \
        libsqlite3-dev:i386 && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-01-09 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add i686-unknown-linux-gnu

ENV PATH="/root/.cargo/bin:$PATH"
