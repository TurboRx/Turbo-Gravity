# ── Stage 1: Build ──────────────────────────────────────────────────────────
FROM rust:1.93-slim-bookworm AS builder

WORKDIR /app

# Install system dependencies needed by some crates (openssl, pkg-config)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependency layer: copy manifests first, build a dummy binary so
# Cargo's dependency fetch is cached separately from source changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main(){}' > src/main.rs \
    && cargo build --release 2>/dev/null || true \
    && rm -rf src

# Now copy the real source and build the release binary
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
