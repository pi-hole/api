FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates \
        gcc libc-dev gcc-powerpc64le-linux-gnu libc-dev-ppc64el-cross && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add powerpc64le-unknown-linux-gnu && \
    apt-get purge -y --autoremove curl ca-certificates

ENV TARGET_CC=powerpc64le-linux-gnu-gcc \
    CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=powerpc64le-linux-gnu-gcc \
    CC_powerpc64le_unknown_linux_gnu=powerpc64le-linux-gnu-gcc