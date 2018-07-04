FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates git \
        gcc libc-dev gcc-arm-linux-gnueabi libc-dev-armel-cross && \
    curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-06-20 && \
    export PATH="/root/.cargo/bin:$PATH" && \
    rustup target add arm-unknown-linux-gnueabi

ENV PATH="/root/.cargo/bin:$PATH"

RUN rustup component add rustfmt-preview

ENV TARGET_CC=arm-linux-gnueabi-gcc \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABI_LINKER=arm-linux-gnueabi-gcc \
    CC_arm_unknown_linux_gnueabi=arm-linux-gnueabi-gcc