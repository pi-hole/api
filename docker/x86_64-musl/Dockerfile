FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates musl-tools
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07
ENV PATH "/root/.cargo/bin:$PATH"

# Install x86_64-musl standard library
RUN rustup target add x86_64-unknown-linux-musl

ENV TARGET_CC=musl-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=musl-gcc \
    CC_x86_64_unknown_linux_musl=musl-gcc
