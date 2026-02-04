# Multi-stage Dockerfile pour Kusanagi
# Stage 1: Build
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy lib to cache dependencies
RUN mkdir src && echo "pub fn main() {}" > src/lib.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false kusanagi

# Copy binary from builder (assuming there's a main.rs or bin target)
COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi 2>/dev/null || echo "No binary found, this is a library"

# Set ownership and permissions
RUN chown -R kusanagi:kusanagi /usr/local/bin/ 2>/dev/null || true

# Switch to non-root user
USER kusanagi

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application (if binary exists)
CMD ["kusanagi"]
