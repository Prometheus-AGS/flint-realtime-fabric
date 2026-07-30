# ── Stage 1: build the admin UI (embedded into the gateway via rust-embed) ────
# frf-gateway `#[derive(RustEmbed)]`s `admin-ui/dist` at compile time (release), so the UI must
# be built before the Rust stage. admin-ui is a pnpm workspace member (root pnpm-workspace.yaml).
# `frf-wasm` is not built here — admin-ui/vite.config.ts substitutes a stub when the wasm artifact
# is absent, so `vite build` succeeds and produces `dist` (the CRDT-wasm feature degrades; the
# embedded static assets the gateway serves are produced).
FROM node:24-slim@sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d AS ui-builder
RUN corepack enable
WORKDIR /build

# Workspace manifests first (cache-friendly install layer). There is no root package.json —
# pnpm-workspace.yaml defines the members; the lockfile lives at the repo root.
COPY pnpm-lock.yaml pnpm-workspace.yaml ./
COPY admin-ui/package.json admin-ui/package.json
COPY sdks/ts/package.json sdks/ts/package.json
COPY sdks/entity-management/package.json sdks/entity-management/package.json
RUN pnpm install --frozen-lockfile

# Sources, then build the workspace SDK deps admin-ui imports (they expose built dist + .d.ts:
# @prometheusags/frf-sdk = sdks/ts, @prometheusags/frf-entity-management = sdks/entity-management),
# then the admin UI itself. Order matters: the siblings must produce dist before admin-ui's tsc.
COPY admin-ui/ admin-ui/
COPY sdks/ts/ sdks/ts/
COPY sdks/entity-management/ sdks/entity-management/
RUN pnpm --dir sdks/ts build \
    && pnpm --dir sdks/entity-management build \
    && pnpm --dir admin-ui build

# ── Stage 2: build the gateway binary (embeds admin-ui/dist) ──────────────────
FROM rust:1.94-bookworm@sha256:6ae102bdbf528294bc79ad6e1fae682f6f7c2a6e6621506ba959f9685b308a55 AS builder

RUN apt-get update && apt-get install -y \
    clang libclang-dev protobuf-compiler pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Cache dependency compilation
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY proto/ proto/
# The admin UI built in stage 1 — must exist BEFORE cargo build (compile-time rust-embed).
COPY --from=ui-builder /build/admin-ui/dist ./admin-ui/dist

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

FROM debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd

RUN apt-get update && apt-get install -y ca-certificates curl libpq5 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/frf-gateway /usr/local/bin/frf-gateway

EXPOSE 8080 9090

ENTRYPOINT ["/usr/local/bin/frf-gateway"]
