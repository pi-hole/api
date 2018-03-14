FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates \
        gcc libc-dev gcc-arm-linux-gnueabihf libc6-dev-armhf-cross && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add arm-unknown-linux-gnueabihf && \
    apt-get purge curl -y --autoremove ca-certificates

ENV PATH="/root/.cargo/bin:$PATH" \
    TARGET_CC=arm-linux-gnueabihf-gcc \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc \
    CC_armv7_unknown_linux_gnueabihf=arm-linux-gnueabihf-gcc
