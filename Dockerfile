# Multi-stage Dockerfile for Rust Marketing Data Crawler
FROM rust:latest as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy Manifests & Source
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*

# Copy compiled binary and static web assets
COPY --from=builder /usr/src/app/target/release/marketing-data-crawler /app/marketing-data-crawler
COPY static /app/static

ENV PORT=8080
ENV RUST_LOG=info
ENV DATABASE_PATH=/app/marketing_leads.db

EXPOSE 8080

CMD ["/app/marketing-data-crawler"]
