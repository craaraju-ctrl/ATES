# Multi-stage build for ATES (Tauri backend + Leptos UI)
FROM rust:1.82 as builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ates-core/Cargo.toml ./crates/ates-core/
COPY crates/ates-agents/Cargo.toml ./crates/ates-agents/
COPY crates/ates-orchestrator/Cargo.toml ./crates/ates-orchestrator/
COPY src-tauri/Cargo.toml ./src-tauri/

# Create dummy main.rs to cache dependencies
RUN mkdir -p crates/ates-core/src crates/ates-agents/src crates/ates-orchestrator/src src-tauri/src && \
    echo "fn main() {}" > crates/ates-core/src/lib.rs && \
    echo "fn main() {}" > crates/ates-agents/src/lib.rs && \
    echo "fn main() {}" > crates/ates-orchestrator/src/main.rs && \
    echo "fn main() {}" > src-tauri/src/main.rs

# Build dependencies
RUN cargo build --release -p ates-ui || true

# Copy actual source
COPY . .

# Build release
RUN cargo build --release -p ates-ui

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libayatana-appindicator3-1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ates-ui /app/ates-ui
COPY --from=builder /app/src-tauri/tauri.conf.json /app/

# Create data directory for memory store
RUN mkdir -p /app/data

ENV RUST_LOG=info

EXPOSE 1420

CMD ["./ates-ui"]