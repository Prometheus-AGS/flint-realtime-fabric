FROM rust:latest AS builder

RUN apt-get update && apt-get install -y \
    clang libclang-dev protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY proto/ proto/

# Build the gateway binary in release mode
RUN cargo build --release -p frf-gateway

FROM debian:trixie-slim

RUN apt-get update && apt-get install -y ca-certificates curl libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/frf-gateway /usr/local/bin/frf-gateway

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/frf-gateway"]
