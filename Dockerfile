# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS builder

WORKDIR /app

# Install system dependencies needed by some crates (openssl, pkg-config)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Pre-fetch dependencies into the Cargo registry cache.
# Copying only the manifests means this layer is rebuilt only when
# Cargo.toml / Cargo.lock change, not when source files change.
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Build the real binary.  Dependencies are already fetched above so this
# only recompiles what changed.
COPY src ./src
RUN cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/turbo-gravity ./turbo-gravity

# The bot reads config.toml from the working directory at runtime.
# Mount or COPY your config.toml here — do NOT embed secrets in the image.
# COPY config.toml ./config.toml

ENV RUST_LOG=info

EXPOSE 8080

CMD ["./turbo-gravity"]
