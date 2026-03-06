FROM rust:1.94-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ── Plan stage: compute the exact dependency fingerprint ──────────────────────
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# ── Build stage ───────────────────────────────────────────────────────────────
FROM chef AS builder

RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev cmake \
    && rm -rf /var/lib/apt/lists/*

# 1. Restore & compile ONLY dependencies (cached as long as Cargo.toml/lock unchanged)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 2. Build your actual code (deps layer above is reused)
COPY . .
RUN cargo build --release --locked

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates openssl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/bot /usr/local/bin/bot
COPY config/domains.json /app/config/domains.json

RUN mkdir -p /app/data
VOLUME /app/data
EXPOSE 8080
CMD ["bot"]