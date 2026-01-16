# Stage 1: Chef - Compute recipe
FROM docker.io/library/rust:1-slim-bookworm AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

# Stage 2: Planner - Create recipe.json
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Builder - Build dependencies and application
FROM chef AS builder
ARG CARGO_INCREMENTAL=1
ENV CARGO_INCREMENTAL=$CARGO_INCREMENTAL
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build --release

# Stage 4: Runtime - Minimal image
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
