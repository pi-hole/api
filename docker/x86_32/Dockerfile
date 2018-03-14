FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc gcc-multilib && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add i686-unknown-linux-gnu

ENV PATH="/root/.cargo/bin:$PATH"
