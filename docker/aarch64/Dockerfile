FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc-aarch64-linux-gnu
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07
ENV PATH "/root/.cargo/bin:$PATH"

# Install target standard library
RUN rustup target add aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc