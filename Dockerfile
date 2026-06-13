# Multi-stage build for tredo (Trading Real-time Edge Decision Optimisation)
FROM rust:1.82 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/tredo-core/Cargo.toml ./crates/tredo-core/
COPY crates/tredo-agents/Cargo.toml ./crates/tredo-agents/
COPY crates/tredo-orchestrator/Cargo.toml ./crates/tredo-orchestrator/
COPY src-tauri/Cargo.toml ./src-tauri/

# Create dummy main.rs to cache dependencies
RUN mkdir -p crates/tredo-core/src crates/tredo-agents/src crates/tredo-orchestrator/src src-tauri/src && \
    echo "fn main() {}" > crates/tredo-core/src/lib.rs && \
    echo "fn main() {}" > crates/tredo-agents/src/lib.rs && \
    echo "fn main() {}" > crates/tredo-orchestrator/src/main.rs && \
    echo "fn main() {}" > src-tauri/src/main.rs

# Build dependencies
RUN cargo build --release -p tredo-ui || true

# Copy actual source
COPY . .

# Build release
RUN cargo build --release -p tredo-ui

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libayatana-appindicator3-1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/tredo-ui /app/tredo-ui
COPY --from=builder /app/src-tauri/tauri.conf.json /app/

# Create data directory for memory store
RUN mkdir -p /app/data

ENV RUST_LOG=info

EXPOSE 1420

CMD ["./tredo-ui"]