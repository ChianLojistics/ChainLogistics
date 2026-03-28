# Build stage
FROM rust:1.75 as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Create empty src directory to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false chainlogistics

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chainlojistic-backend /usr/local/bin/chainlojistic-backend

# Copy migrations
COPY --from=builder /app/migrations ./migrations

# Create necessary directories
RUN mkdir -p /backups && \
    chown -R chainlogistics:chainlogistics /app

# Switch to non-root user
USER chainlogistics

# Expose port
EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3001/health || exit 1

# Run the application
CMD ["chainlojistic-backend"]
