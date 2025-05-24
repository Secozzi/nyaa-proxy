# Leveraging the pre-built Docker images with
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

RUN apk add openssl-dev

# Build application
COPY . .
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --bin nyaa-proxy

# We do not need the Rust toolchain to run the binary!
FROM alpine:latest AS runtime
WORKDIR /app

RUN apk add libgcc

COPY --from=chef /app/target/release/nyaa-proxy /usr/local/bin
ENTRYPOINT ["/usr/local/bin/nyaa-proxy"]
