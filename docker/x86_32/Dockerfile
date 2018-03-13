FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc gcc-multilib
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07
ENV PATH "/root/.cargo/bin:$PATH"

# Install x86_32 standard library
RUN rustup target add i686-unknown-linux-gnu
