FROM rust:1.93-slim AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm src/main.rs

COPY src ./src
COPY static ./static
COPY build.rs ./
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*

# Install kubectl - robust version fetch
RUN export KUBECTL_VERSION=$(curl -L -s https://dl.k8s.io/release/stable.txt | tr -d '\n' | tr -d '\r') && \
    curl -L "https://dl.k8s.io/release/${KUBECTL_VERSION}/bin/linux/amd64/kubectl" -o /usr/local/bin/kubectl && \
    chmod +x /usr/local/bin/kubectl

COPY --from=builder /app/target/release/kusanagi /usr/local/bin/kusanagi
COPY --from=builder /app/static /app/static
RUN useradd -r -s /bin/false kusanagi && chown -R kusanagi:kusanagi /app

WORKDIR /app
USER kusanagi
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["/usr/local/bin/kusanagi"]
