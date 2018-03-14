FROM debian:stretch

# Install Rust
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates gcc libc-dev
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly-2018-03-07
ENV PATH "/root/.cargo/bin:$PATH"
