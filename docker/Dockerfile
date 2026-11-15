# -------------------------------------------------------------
# Stage 1: Rust Builder Environment
# -------------------------------------------------------------
FROM rust:1.80-alpine AS builder

RUN apk add --no-cache musl-dev gcc git

WORKDIR /usr/src/specforge

# Copy manifests
COPY Cargo.toml Cargo.lock* ./
COPY crates ./crates
COPY frontend ./frontend

# Compile release binary with static musl linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# -------------------------------------------------------------
# Stage 2: Ultra-Lightweight Production Runtime (< 20 MB)
# -------------------------------------------------------------
FROM alpine:3.20 AS runtime

LABEL maintainer="SpecForge Team <team@specforge.dev>"
LABEL description="Ultra-fast Rust Developer Engine, Mock Server & Boundary Fuzzer"

# Create non-root user and mount directories
RUN addgroup -S specforge && adduser -S specforge -G specforge && \
    mkdir -p /specs /output /data && \
    chown -R specforge:specforge /specs /output /data

WORKDIR /home/specforge

# Copy compiled static binary from builder
COPY --from=builder /usr/src/specforge/target/x86_64-unknown-linux-musl/release/specforge /usr/local/bin/specforge

# Copy entrypoint script
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER specforge

# Expose Studio UI (:4000)
EXPOSE 4000

VOLUME ["/specs", "/output", "/data"]

ENV SPECFORGE_HOST=0.0.0.0 \
    SPECFORGE_PORT=4000 \
    RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["serve"]
