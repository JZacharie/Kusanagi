FROM debian:trixie-slim AS runner
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && chmod 644 /etc/ssl/certs/* \
    && chmod 755 /etc/ssl/certs

# Install kubectl - robust version fetch
ARG TARGETARCH
RUN export KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt | tr -d '\n' | tr -d '\r') && \
    curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/${TARGETARCH}/kubectl" -o /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl

WORKDIR /app
# Invalidate cache on static changes (v2 migration - modular k8s)
ARG STATIC_VERSION=v2
# Ensure old monolithic k8s.js is removed (migration to modular structure)
RUN rm -f static/js/k8s.js 2>/dev/null || true
COPY static ./static
# Verify new structure exists
RUN ls -la /app/static/js/k8s/ && test -f /app/static/js/k8s/main.js || exit 1
RUN useradd -r -s /bin/false kusanagi && chown -R kusanagi:kusanagi /app

# Ensure CA certificates are accessible
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/kusanagi"]

# --- CI Build (uses pre-built binary) ---
FROM runner AS release-ci
# Force cache invalidation when static changes (use --build-arg CACHE_BUST=$(date +%s))
ARG CACHE_BUST=0
ARG PREBUILT_BINARY
# Re-copy static to ensure fresh version (overrides runner stage cache)
COPY static ./static
# Verify static structure
RUN ls -la /app/static/js/k8s/ && test -f /app/static/js/k8s/main.js || (echo "Missing k8s modules!" && exit 1)
COPY ${PREBUILT_BINARY} /usr/local/bin/kusanagi
USER kusanagi

# --- Full Build (local or fallback) ---
FROM rust:1.93-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev git curl unzip && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm src/main.rs

COPY src ./src
COPY static ./static
COPY build.rs ./
COPY scripts ./scripts
RUN chmod +x scripts/*.sh
RUN cargo build --release

FROM runner AS release
COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi
USER kusanagi
