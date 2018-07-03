FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates libc-dev musl-tools git && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-06-20 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add x86_64-unknown-linux-musl

ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup component add rustfmt-preview

ENV TARGET_CC=musl-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=musl-gcc \
    CC_x86_64_unknown_linux_musl=musl-gcc
