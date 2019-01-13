FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev git \
        libsqlite3-dev && \
    rm -rf /var/lib/apt/lists/* && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2019-01-09 && \
    export PATH="/root/.cargo/bin:$PATH"

ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup component add rustfmt clippy
