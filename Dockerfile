FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    clang libclang-dev protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY proto/ proto/

# CARGO_FEATURES: pass additional Cargo features at build time.
# Example: --build-arg CARGO_FEATURES=dev-endpoints (enables /dev/* routes for CI).
# Leave empty for production images.
ARG CARGO_FEATURES=""

# Build the gateway binary in release mode
RUN if [ -n "$CARGO_FEATURES" ]; then \
        cargo build --release -p frf-gateway --features "$CARGO_FEATURES"; \
    else \
        cargo build --release -p frf-gateway; \
    fi

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates curl libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/frf-gateway /usr/local/bin/frf-gateway

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/frf-gateway"]
