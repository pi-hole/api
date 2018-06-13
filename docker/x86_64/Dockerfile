FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev git && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07 && \
    export PATH="/root/.cargo/bin:$PATH"

ENV PATH="/root/.cargo/bin:$PATH"