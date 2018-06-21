FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev gcc-aarch64-linux-gnu libc-dev-arm64-cross && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-06-20 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add aarch64-unknown-linux-gnu

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=aarch64-linux-gnu-gcc \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc