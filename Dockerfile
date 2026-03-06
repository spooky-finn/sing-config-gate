# Build stage
FROM rust:1.85-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests for dependency caching
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency build
RUN mkdir -p src/bin && \
    echo "fn main() {}" > src/main.rs && \
    for bin in gen_node_config deploy; do \
        mkdir -p src/bin/$bin && \
        echo "fn main() {}" > src/bin/$bin/main.rs; \
    done && \
    mkdir -p src/adapters src/db src/domain src/service src/utils && \
    for f in adapters db domain service utils; do \
        echo "pub mod $f {}" >> src/lib.rs; \
    done

# Build dependencies only (cached layer)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/bot .

# Copy config files
COPY config/domains.json ./config/

# Create .env file location
RUN mkdir -p /app/.env

EXPOSE 8080

CMD ["./bot"]
