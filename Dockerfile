# --- Runner ---
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

WORKDIR /app
RUN useradd -r -s /bin/false kusanagi && chown -R kusanagi:kusanagi /app

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/kusanagi"]

# --- CI Build (uses pre-built binary) ---
FROM runner AS release-ci
ARG PREBUILT_BINARY
COPY static ./static
COPY ${PREBUILT_BINARY} /usr/local/bin/kusanagi
RUN ls -la /app/static/js/k8s/ && test -f /app/static/js/k8s/main.js \
    && chown -R kusanagi:kusanagi /app/static \
    && chmod +x /usr/local/bin/kusanagi
USER kusanagi

# --- Full Build (optimized with cargo-chef) ---
FROM lukemathwalker/cargo-chef:latest-rust-1.93.1-slim AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the layer that will be cached
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin kusanagi

FROM runner AS release
COPY static ./static
COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi
RUN chown -R kusanagi:kusanagi /app
USER kusanagi
