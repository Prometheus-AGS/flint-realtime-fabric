# KBD Constraints — flint-realtime-fabric

> The canonical constraint set the per-change QA gate validates against. Read by
> `/kbd-execute`'s QA step (and the `qa-gate.sh` helper) after a change reaches DONE,
> before archive. Source of truth: `CLAUDE.md`, `docs/PROMETHEUS-BASE-RULES.md`, and
> `~/.claude/rules/rust/*`. Severity drives the gate: **BLOCKING** fails the change
> (→ mark BLOCKED + refine); HIGH/MEDIUM are reported but do not block.

## Legend

- **BLOCKING** — a violation fails the QA gate; the change is marked BLOCKED in
  `progress.json` and must be refined before archive.
- **HIGH / MEDIUM** — reported in the refinement log; reviewed, not auto-blocking.

---

## Architecture (BLOCKING)

- **A1 — Dependency direction.** Nothing in `frf-domain` or `frf-app`/`frf-ports` may
  import an adapter crate (`frf-broker-*`, `frf-authz-*`, `frf-store-*`, `frf-media-*`,
  `frf-bridge-*`, `frf-identity-*`, `frf-policy-*`, `frf-crdt`, `frf-postgres-cdc`).
  Composition happens only in `frf-gateway`.
  *Check:* the `[dependencies]` of `frf-domain` and `frf-app` contain no adapter crate.
- **A2 — One port per adapter.** Each `frf-*` adapter implements exactly one port trait
  and does not reach into another adapter crate.
- **A3 — Proto is frozen.** No edit to `proto/flint/v1/*.proto` (frozen at `proto-v1`).
  A wire-breaking change is a NEW proto version. Non-Rust SDKs generate from the source
  proto — no forked copy committed under `sdks/`.

## Rust quality gates (BLOCKING)

- **R1 — Compiles.** `cargo check --workspace` is clean.
- **R2 — Clippy.** `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic`
  is clean, plus the `--features frf-gateway/dev-endpoints` and `--tests` passes.
- **R3 — No library `unwrap()`/`expect()`.** `clippy::unwrap_used` / `expect_used` are
  denied at the workspace level; they are allowed only in tests (`clippy.toml`). `anyhow`
  only at binary edges (`frf-gateway`, `frf-cli`).
- **R4 — Formatted.** `cargo fmt --check --all` is clean.
- **R5 — File size.** No file over **500 lines** (any language). Split into a directory
  module when approaching it.

## Rust idioms (HIGH)

- **I1 — `#[non_exhaustive]`** on public enums.
- **I2 — Newtype IDs** (`#[repr(transparent)]`) over bare `String` for entity IDs.
- **I3 — `tracing` spans** across every port-boundary call.
- **I4 — MSRV pinned** in the workspace manifest; semver discipline on `frf-domain` and
  SDK crates.

## Security (BLOCKING)

- **S1 — No hardcoded secrets** (API keys, tokens, signing secrets) in any committed
  file. Secrets come from env / a secret manager; validated present at startup.
- **S2 — No auth bypass in a release artifact.** `DEV_NO_AUTH` / `dev-endpoints` are
  compiled out of the production image; never enabled in `compose.yml`.
- **S3 — No sensitive logging.** Never log JWT payloads, relation tuples, or tenant
  identifiers as credentials. (`tenant_id` as a structured observability field is OK.)
- **S4 — Tenant isolation** is enforced at Keto plus the app-layer tenant-equality guard;
  never trust unverified claims downstream.

## Tests & docs (HIGH)

- **T1 — Tests for new behavior.** New logic carries unit/integration tests (AAA); target
  80%+ where practical. Live-service tests gate on env vars and skip cleanly when unset.
- **T2 — Honest surface.** No "healthy but does nothing": a plane/endpoint is either
  functional or explicitly labeled unimplemented/deferred. No doc drift — docs match code.

## Frontend / TypeScript (BLOCKING where the change touches `admin-ui/` or `sdks/ts`)

- **F1 — No `any` types.** TSX over JSX. Components render; hooks coordinate; stores own
  state; services call APIs — no component fetches data directly.

## Spec-backend hygiene (BLOCKING for OpenSpec changes)

- **P1 — Valid spec delta.** Each change carries a `specs/<capability>/spec.md` delta
  (`## ADDED/MODIFIED/...` + a MUST/SHALL body line + `#### Scenario:` blocks) so
  `openspec validate <id>` passes before archive.

---

## Per-change applicability

Not every constraint applies to every change. The gate evaluates the subset relevant to
the files a change touched:

| Change kind | BLOCKING subset |
|-------------|-----------------|
| Rust code | A1–A3, R1–R5, S1–S4, P1 |
| Config / compose | S1, S2, P1 |
| Docs-only | T2 (no drift), P1 |
| Frontend / TS | F1, R5 (size), P1 |
| Tooling / KBD | P1 |
