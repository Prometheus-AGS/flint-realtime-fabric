# p13-c001 — Replace `cfg(debug_assertions)` with `dev-endpoints` Cargo feature

## Summary

The `/dev/inject-federation-event` endpoint (and the `dev.rs` module) are
currently gated by `#[cfg(debug_assertions)]`. The gateway Dockerfile builds
with `--release`, which disables `debug_assertions`, so the endpoint is absent
in the compose image. Phase 6 federation smoke tests POST to this endpoint and
receive 404 in every Stage 10 run.

Replace `#[cfg(debug_assertions)]` with a dedicated `dev-endpoints` Cargo
feature. Enable the feature in `compose.yml` via a Docker build arg so the
compose image includes the endpoint while production images do not.

## Files to change

- `crates/frf-gateway/Cargo.toml` — add `dev-endpoints` feature
- `crates/frf-gateway/src/routes/dev.rs` — replace `#[cfg(debug_assertions)]` with `#[cfg(feature = "dev-endpoints")]`
- `crates/frf-gateway/src/lib.rs` — replace `#[cfg(debug_assertions)]` gate on route registration
- `Dockerfile` — add `ARG CARGO_FEATURES=""` + use it in `cargo build`
- `compose.yml` — add `build.args.CARGO_FEATURES: dev-endpoints` to gateway service

## Specification

### `crates/frf-gateway/Cargo.toml`

Add under `[features]`:

```toml
[features]
default = []
dev-endpoints = []
```

### `crates/frf-gateway/src/routes/dev.rs`

Replace every occurrence of:
```rust
#[cfg(debug_assertions)]
```
with:
```rust
#[cfg(feature = "dev-endpoints")]
```

(There is one module-level gate on line 7: `#[cfg(debug_assertions)] pub mod inject {`)

### `crates/frf-gateway/src/lib.rs`

Find the route registration block that registers `inject_federation_event` under
`#[cfg(debug_assertions)]` and change it to `#[cfg(feature = "dev-endpoints")]`.

### `Dockerfile`

```dockerfile
# Before the RUN cargo build line, add:
ARG CARGO_FEATURES=""

# Change:
RUN cargo build --release -p frf-gateway
# To:
RUN cargo build --release -p frf-gateway \
    $([ -n "$CARGO_FEATURES" ] && echo "--features $CARGO_FEATURES" || true)
```

### `compose.yml`

Under the `gateway` service, add:

```yaml
  gateway:
    build:
      context: .
      dockerfile: Dockerfile
      args:
        CARGO_FEATURES: dev-endpoints
```

## Acceptance criteria

1. `cargo build --release -p frf-gateway` (no features) does not expose
   `/dev/inject-federation-event`.
2. `cargo build --release -p frf-gateway --features dev-endpoints` exposes
   the endpoint and returns `202` on a valid POST.
3. `docker compose build` produces an image with the endpoint present.
4. `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`
   passes with the feature enabled and disabled.
