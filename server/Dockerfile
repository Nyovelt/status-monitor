# Build stage
FROM rust:bookworm AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/server /app/server

# Create data directory
RUN mkdir -p /app/data

ENV DATABASE_URL=sqlite:/app/data/monitor.db
ENV RUST_LOG=info

EXPOSE 8080

CMD ["/app/server"]
