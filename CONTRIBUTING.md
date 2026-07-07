# Contributing to flint-realtime-fabric

Thanks for contributing. This guide covers the local workflow and the quality gates
that CI enforces — get these green locally before opening a PR.

## Prerequisites

- Rust (see `rust-version` in `Cargo.toml` for the pinned MSRV) + `cargo fmt`, `cargo clippy`
- Node 24 + `pnpm` (admin UI, TS SDK)
- Docker + Docker Compose (integration / Layer-3 E2E)
- For SDK regeneration: `dotnet` (C#), Flutter/Dart + `uniffi-bindgen-dart` (Dart),
  `uniffi-bindgen` (Swift/Kotlin — via the workspace `uniffi-bindgen` crate)

## Architecture rules (non-negotiable)

- **Dependency direction:** Domain (`frf-domain`) ← App (`frf-app`/`frf-ports`) ←
  Adapters (`frf-*`) ← Interface (`frf-gateway`). Nothing in domain/app imports an
  adapter. One port per adapter. Composition happens only in `frf-gateway`.
- **The proto is the contract.** `proto/flint/v1` is frozen at `proto-v1`. A
  wire-breaking change is a NEW proto version, never an edit. SDKs generate from the
  source-of-truth proto — never commit a forked copy.
- **File size:** no file over 500 lines; split into a directory module.
- See `CLAUDE.md` and `docs/PROMETHEUS-BASE-RULES.md` for the full ruleset.

## Rust quality gates (CI-enforced)

```bash
cargo fmt --check --all
cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic
cargo clippy --workspace --lib --bins --features frf-gateway/dev-endpoints -- -D warnings -W clippy::pedantic
cargo clippy --workspace --tests -- -D warnings -W clippy::pedantic
cargo test --workspace
```

- **No `unwrap()` / `expect()` in library code** — `clippy::unwrap_used`/`expect_used`
  are denied at the workspace level (`clippy.toml` allows them in tests). Use
  `thiserror` for library errors; `anyhow` only at binary edges (`frf-gateway`,
  `frf-cli`).
- `#[non_exhaustive]` on public enums; newtype IDs; `tracing` spans across port
  boundaries.

## Frontend / SDK gates

```bash
cd admin-ui && pnpm typecheck && pnpm lint    # React 19 admin UI
cd sdks/ts   && pnpm typecheck                # TS SDK
cd sdks/go   && go build ./... && go vet ./... && gofmt -l .
cd sdks/csharp && dotnet build
```

## Tests

- Unit tests inline (`#[cfg(test)]`) / integration in `tests/`. Follow AAA.
- Integration suites that need live services are gated behind env vars
  (`FRF_GATEWAY_GRPC_URL`, `SKIP_INTEGRATION`, `GATEWAY_URL`) and skip cleanly when unset.
- Layer-3 E2E runs under Dagger (DinD) — see the Makefile.

## Commits & PRs

- Conventional commits: `feat|fix|refactor|docs|test|chore|perf|ci: <summary>`.
- Analyze the full diff (`git diff <base>...HEAD`) and include a test plan in the PR.
- Ensure all CI gates pass and the branch is up to date before requesting review.

## Reporting security issues

Do NOT open a public issue for vulnerabilities — see [`SECURITY.md`](SECURITY.md).
