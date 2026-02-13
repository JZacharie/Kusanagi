FROM rust:1.93-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm src/main.rs

COPY src ./src
COPY static ./static
COPY build.rs ./
COPY scripts ./scripts
RUN chmod +x scripts/*.sh
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && chmod 644 /etc/ssl/certs/* \
    && chmod 755 /etc/ssl/certs

# Install kubectl - robust version fetch
ARG TARGETARCH
RUN export KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt | tr -d '\n' | tr -d '\r') && \
    curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/${TARGETARCH}/kubectl" -o /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl

COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi
COPY --from=builder /app/static /app/static
RUN useradd -r -s /bin/false kusanagi && chown -R kusanagi:kusanagi /app

WORKDIR /app

# Ensure CA certificates are accessible
ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

USER kusanagi
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/kusanagi"]
