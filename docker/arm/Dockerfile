FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev gcc-arm-linux-gnueabi libc-dev-armel-cross
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07
ENV PATH "/root/.cargo/bin:$PATH"

# Install arm standard library
RUN rustup target add arm-unknown-linux-gnueabi

ENV TARGET_CC=arm-linux-gnueabi-gcc \
    CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABI_LINKER=arm-linux-gnueabi-gcc \
    CC_arm_unknown_linux_gnueabi=arm-linux-gnueabi-gcc