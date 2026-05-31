# syntax=docker/dockerfile:1
# Multi-stage build for i-rs-claw
# Build args:
#   FEATURES  - Cargo features (default: "dashboard,sqlite")
#   RUST_IMG  - Rust base image tag (default: "slim-bookworm")

ARG RUST_IMG=slim-bookworm

# ============================================================
# Stage 1: Planner — compute dependency recipe via cargo-chef
# ============================================================
FROM rust:${RUST_IMG} AS planner
WORKDIR /app
RUN cargo install cargo-chef --locked

# Copy workspace manifests (all members required for cargo metadata)
COPY Cargo.toml Cargo.lock ./
COPY crates crates/

# Strip source code — keep only Cargo.toml for metadata resolution
RUN find crates -type f ! -name 'Cargo.toml' -delete

RUN cargo chef prepare --recipe-path recipe.json

# ============================================================
# Stage 2: Cacher — build & cache dependencies
# ============================================================
FROM rust:${RUST_IMG} AS cacher
WORKDIR /app
RUN cargo install cargo-chef --locked

COPY Cargo.toml Cargo.lock ./
COPY crates crates/
RUN find crates -type f ! -name 'Cargo.toml' -delete

COPY --from=planner /app/recipe.json recipe.json

ARG FEATURES="dashboard,sqlite"
RUN cargo chef cook --release --features "$FEATURES" -p i-rs-claw --recipe-path recipe.json

# ============================================================
# Stage 3: Builder — compile the actual binary
# ============================================================
FROM rust:${RUST_IMG} AS builder
WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Full source code (excluding artifacts)
COPY Cargo.toml Cargo.lock ./
COPY crates crates/

# Reuse cached dependency builds
COPY --from=cacher /app/target target/
COPY --from=cacher /usr/local/cargo /usr/local/cargo/

ARG FEATURES="dashboard,sqlite"
RUN cargo build --release --features "$FEATURES" -p i-rs-claw && \
    cp target/release/i-rs-claw /usr/local/bin/ && \
    strip /usr/local/bin/i-rs-claw

# ============================================================
# Stage 4: Runtime — minimal Debian image
# ============================================================
FROM debian:bookworm-slim AS runtime
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/bin/i-rs-claw /usr/local/bin/i-rs-claw

ENV CONFIG_DIR=/data/i-rs
VOLUME /data/i-rs

ENTRYPOINT ["i-rs-claw"]
CMD ["tui"]
