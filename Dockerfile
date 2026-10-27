# Multi-stage build
FROM rust:1.75-slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* && \
    useradd -m -u 1001 appuser

COPY --from=builder --chown=appuser:appuser /app/target/release/pyroboreplay /usr/local/bin/

USER appuser
ENTRYPOINT ["pyroboreplay"]
