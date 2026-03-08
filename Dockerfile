FROM debian:trixie-slim AS runner
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && chmod 644 /etc/ssl/certs/* \
    && chmod 755 /etc/ssl/certs \
    # Install kubectl
    && export KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt | tr -d '\n' | tr -d '\r') \
    && curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/${TARGETARCH}/kubectl" -o /usr/local/bin/kubectl \
    && chmod +x /usr/local/bin/kubectl

# Create base app structure (without static - will be added later)
WORKDIR /app
RUN useradd -r -s /bin/false kusanagi && chown -R kusanagi:kusanagi /app

# Ensure CA certificates are accessible
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/kusanagi"]

# --- CI Build (uses pre-built binary) ---
# Start from runner but explicitly remove any cached static first
FROM runner AS release-ci
ARG PREBUILT_BINARY

# Copy fresh static from build context
COPY static ./static
COPY ${PREBUILT_BINARY} /usr/local/bin/kusanagi

# Verify and set permissions
RUN ls -la /app/static/js/k8s/ && test -f /app/static/js/k8s/main.js \
    && chown -R kusanagi:kusanagi /app/static \
    && chmod +x /usr/local/bin/kusanagi

USER kusanagi

# --- Full Build (local or fallback) ---
FROM rust:1.93-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev git curl unzip && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm src/main.rs

COPY src ./src

COPY build.rs ./
COPY scripts ./scripts
RUN chmod +x scripts/*.sh
RUN cargo build --release

# Stage 4: Runtime - Minimal image (standard build)
FROM gcr.io/distroless/cc-debian12
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/kusanagi /app/kusanagi

# Copy static files
COPY --from=builder /app/static /app/static

# Expose port
EXPOSE 8080

# Run application
CMD ["/app/kusanagi"]
