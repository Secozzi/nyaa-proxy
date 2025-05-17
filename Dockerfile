# Leveraging the pre-built Docker images with
# cargo-chef and the Rust toolchain
FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apk add openssl-dev

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build --release --bin nyaa-proxy

# We do not need the Rust toolchain to run the binary!
FROM alpine:latest AS runtime
WORKDIR /app

RUN apk add libgcc

COPY --from=builder /app/target/release/nyaa-proxy /usr/local/bin
ENTRYPOINT ["/usr/local/bin/nyaa-proxy"]
