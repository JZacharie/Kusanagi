# Dockerfile pour Kusanagi - Application Runtime
FROM rust:1.88-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "pub fn main() {}" > src/lib.rs && echo "fn main() {}" > src/main_complete.rs
RUN cargo build --release && rm -rf src

# Copy source code
COPY src ./src

# Build the application binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false kusanagi

# Set environment variables
ENV RUST_LOG=debug
ENV KUSANAGI_HOST=0.0.0.0
ENV KUSANAGI_PORT=8080

# Copy binary from builder
COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi

# Set ownership and permissions
RUN chown kusanagi:kusanagi /usr/local/bin/kusanagi
RUN chmod +x /usr/local/bin/kusanagi

# Create working directory
RUN mkdir -p /app && chown kusanagi:kusanagi /app
WORKDIR /app

# Switch to non-root user
USER kusanagi

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application with verbose output
CMD ["/usr/local/bin/kusanagi"]
