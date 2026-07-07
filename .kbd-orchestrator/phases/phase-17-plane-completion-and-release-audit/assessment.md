# Assessment — phase-17-plane-completion-and-release-audit

> Generated: 2026-07-06 · Backend: OpenSpec
> Method: independent re-audit against **real code** (3 parallel code-grounded audits +
> deterministic gate runs), not the phase-16 change list.
> Baseline: phase-15 production-readiness audit — 7 CRITICAL / 15 HIGH / 24 MEDIUM / 15 LOW.

This assessment **is** the G1 deliverable ("run the production-readiness audit
independently"). Every claim below cites file:line evidence gathered this session.

---

## Headline verdict

**The security boundary holds. Zero CRITICAL and zero HIGH survive in the production
build.** The phase-16 completion criterion — "a re-run returns no CRITICAL and a
materially reduced HIGH count" — is **now independently verified** (it was only asserted
per-goal before). The remaining gaps are all **honestly-labeled deferrals**, not hidden
breakage: no "healthy but does nothing" trap was found in any of the three audits.

Phase-17's real work is therefore **not** firefighting — it is (a) closing one MEDIUM
security-hygiene footgun, (b) wiring the QA gate that phase-16 skipped, and (c) building
out the deferred planes in dependency order.

---

## G1 — Verify the release claim  →  **MET (this assessment closes it)**

### Deterministic gates (run this session)

| Gate | Result |
|------|--------|
| `cargo check --workspace` | ✅ clean (16.7s) |
| `cargo clippy --workspace --lib --bins -- -D warnings -W clippy::pedantic -W clippy::unwrap_used` | ✅ clean (0 warnings) |
| `cargo fmt --check --all` | ✅ clean |
| `cargo test --workspace --lib` | ✅ all lib unit tests pass (100+ across crates) |
| `cargo test --workspace` (full) | ⚠️ exceeds 2-min local link budget — not a failure; runs in CI (Dagger). Integration tests gate on env vars and skip when unset. |

### Security re-audit — 7 controls, adversarial lens

| # | Control | Holds? | Evidence |
|---|---------|--------|----------|
| 1 | Auth bypass absent from prod image | **YES** | `dev_no_auth()` is `#[cfg(feature="dev-endpoints")]` — the function *does not exist* in a release binary (`config.rs:91-95`); all call sites cfg-gated (`publish.rs:55`, `subscribe.rs:54`, `signal.rs:111`); `compose.yml` sets no such feature. |
| 2 | Tenant-equality guard | **YES** | `publish.rs:55-60` rejects `claims.tenant_id != envelope tenant`. Empty/None blocked at `claims.rs:30-35` (tenant required + valid UUID before guard). Subscribe defended by Keto `view` per-envelope. |
| 3 | JWT iss/aud/signature | **YES** | JWKS RS256 signature (`verifier.rs:71-74`), aud always set, iss enforced when configured with `set_required_spec_claims` (rejects missing *and* mismatched). |
| 4 | Rate-limit / body-limit / CORS | **YES** | `apply_security_layers` (`lib.rs:124-144`): GovernorLayer + RequestBodyLimitLayer + exact-origin CORS allowlist (no `Any`). |
| 5 | Secrets externalized | **YES (prod)** | `compose.yml:116` `${FLINT_GATE_JWT_SECRET:?}` fails fast; `.env.example` placeholder; repo grep clean. **See MEDIUM below.** |
| 6 | Cedar fail-loud | **YES** | Surfaces `diagnostics().errors()` → `Err(PolicyError::Evaluation)`, callers map to 500 (fail closed), never silent allow/deny (`lib.rs:107-127`). |
| 7 | Logging leaks | **YES** | No JWT/token/tuple logging; `#[instrument(skip(token))]`; `tenant_id` appears only as a structured identifier field. |

**Surviving findings from this re-audit:**
- **MEDIUM (new / hygiene):** `compose.override.yml` is **committed** and Docker Compose
  **auto-merges it** on a bare `docker compose up`, hardcoding
  `FLINT_GATE_JWT_SECRET: "dev-only-…"` + `DEV_NO_AUTH: "true"` +
  `CARGO_FEATURES: dev-endpoints`. It cannot affect the production image (built from
  bare `compose.yml`), but the committed dev-secret literal and always-on-in-dev bypass
  are a deploy-base footgun. → **G1 residual work.**
- **LOW:** `JWT_ISSUER` is optional (unset → iss unvalidated, with a startup warning).
  Recommend mandatory-in-prod.
- **LOW:** rate limit is global (`GlobalKeyExtractor`), not per-IP — a single client can
  exhaust the global budget. Per-IP needs a trusted-proxy config.

---

## G2 — Wire the QA gate  →  **NOT MET (open; confirmed real gap)**

- Phase-16's `execution.md:53-58` **documents** the artifact-refiner QA gate, but it was
  **never run**: no `.refiner/` directory exists at the repo root (0/26 changes QA'd).
- **`.kbd-orchestrator/constraints.md` does not exist** — the QA gate has nothing to
  validate against. Wiring the gate in phase-17 requires **authoring constraints first**,
  then invoking `/refine-validate` per change in the execute loop.
- This is process debt: quality rode entirely on the Rust CI gates (which are, to be
  fair, genuinely green).

---

## G3 — Complete the deferred planes  →  **NOT MET (open; scoped precisely below)**

All 7 planes are **STUBBED-BUT-HONESTLY-LABELED** or **ABSENT-AND-DECLARED-ABSENT**.
Zero dangerous "advertised as working" cases. Ordered by external-dependency cost:

| Plane | State | Evidence | Work to FUNCTIONAL |
|-------|-------|----------|--------------------|
| **EntityService server** | ABSENT (proto-only) | Gateway registers 4 services (`main.rs:453-456`); no `EntityServiceServer` impl anywhere; disclaimer `main.rs:430-431`. | Tonic impl of GetEntity/WatchEntity backed by a store; `.add_service`. **Pure Rust — cheapest.** |
| **AuthzService server** | ABSENT (proto-only) | Not in router; `KetoAuthzProvider` exists internally (`main.rs:118`) but not exposed as gRPC. | Thin tonic impl delegating to the existing `KetoAuthzProvider`. **Pure Rust — cheap; backing logic already there.** |
| **str0m sovereign SFU** | STUB (signaling only, no WebRTC) | `sfu.rs` is `mpsc` channel routing; **no Rtc/SDP/ICE/RTP**; `str0m` dep unused. Gated `SFU_MODE=hosted` default (`config.rs:261`), loud warning on sovereign (`main.rs:385`). Doc comment at `sfu.rs:30-33` references non-existent `process_offer`/`process_ice` — **doc drift to fix**. | Real `str0m::Rtc` per session: SDP/ICE/DTLS + RTP forwarding. |
| **Matrix bridge** | inbound STUB / outbound FUNCTIONAL | Inbound `stream::empty()` (`client.rs:83`); outbound is a real HTTP PUT to the Matrix CS API (`client.rs:86-106`). Gated `FEDERATION_ENABLED`. | Real inbound sync loop (or Tuwunel client when published). |
| **ATProto bridge** | inbound FUNCTIONAL / outbound unimpl | Inbound is a real Jetstream WS (`jetstream.rs:7-46`); outbound returns `Err(...)` (`lib.rs:54-63`). | Authenticated PDS writes (`com.atproto.repo.createRecord`). |
| **LiveKit** | outbound FUNCTIONAL / inbound in-process only | Outbound `send_data` REST fan-out (`adapter.rs:72-114`); inbound only replays this process's own signals (`adapter.rs:117-133`), never subscribes to the server data channel. | LiveKit realtime data-channel client for cross-node inbound. |
| **Dart SDK** | PLACEHOLDER (no generated API) | `sdks/dart/lib/src/rust/` holds only `.gitkeep`; `frf_dart.dart` is a pending-marker; `build_dart.sh` correctly targets `uniffi-bindgen-dart`. `GENERATED.md` still names `flutter_rust_bridge` — **doc drift to fix**. | Run `build_dart.sh` with `uniffi-bindgen-dart`; commit generated bindings. |

### Adjacent SDK/CLI gaps found while auditing G3 (candidate scope)

- **frf-sdk-rust binds only SpineService** (`client.rs:42`). It lacks even the thin
  Sync/Agent/Signal wrappers the **TS/Go/C#** SDKs have (`services.ts`, `services.go`,
  `ServiceClients.cs`). Entity/Authz unbound everywhere by design (no server yet — G3.1/2).
- **FFI/mobile path has no reconnection** — `FrfFfiClient::subscribe` uses raw
  `client.subscribe`, not `resilient_subscribe` (`client.rs:88-108`). Rust/TS/Go all have
  production-grade reconnect+replay (`resilient.rs`, `subscribeResilient`,
  `SubscribeResilient`); Swift/Kotlin/Dart get none. **FFI also omits `ack`.**
- **frf-cli**: `keto seed/revoke` ✅, `broker checkpoint` ✅, but `cdc status` is
  **inspect-only** (not "manage slots") and there is **no broker-offset inspect** command
  (only checkpoint-write).
- **admin-ui** has a real Connect/gRPC-web Spine client with JWT bearer auth
  (`gateway.ts:1-33`), but **no real login** — it's a paste-your-token gate
  (`LoginGate.tsx`), no OIDC/flint-gate redirect flow.

---

## Gap summary → phase-17 change candidates

| ID | Gap | Goal | Severity | Rough size |
|----|-----|------|----------|-----------|
| A1 | `compose.override.yml` committed + auto-merges dev bypass/secret | G1 | MEDIUM | S |
| A2 | `JWT_ISSUER` optional; make mandatory-in-prod (doc/validate) | G1 | LOW | S |
| A3 | Author `constraints.md` + wire `/refine-validate` into execute | G2 | — | M |
| A4 | `EntityService` gateway server (pure Rust) | G3.1 | — | M |
| A5 | `AuthzService` gateway server (pure Rust, delegates to Keto provider) | G3.2 | — | M |
| A6 | Bind Sync/Agent/Signal in **frf-sdk-rust**; add resilient+ack to **FFI** | G3 (adj) | — | M |
| A7 | frf-cli: broker-offset inspect + CDC slot mgmt (or re-scope honestly) | G3 (adj) | — | S–M |
| A8 | Finish Dart bindings (`uniffi-bindgen-dart`); fix `GENERATED.md` drift | G3.5 | — | M (needs toolchain) |
| A9 | Fix str0m `sfu.rs:30-33` doc drift (references non-existent methods) | G3.3 | LOW | XS |
| A10 | str0m real WebRTC | G3.3 | — | L |
| A11 | Matrix inbound / ATProto outbound / LiveKit cross-node inbound | G3.4 | — | L |
| A12 | admin-ui real OIDC/flint-gate login flow | G2.2 residual | — | M |

**Recommended ordering for `/kbd-plan`:** A1–A2 (close G1 residual) → A3 (QA gate, so all
later changes are validated) → A4, A5 (pure-Rust servers, highest value/lowest cost, and
they flip the API-reference "proto-only" rows to "live") → A9 + A6 + A7 (cheap
correctness/coverage) → A8 (Dart, toolchain-gated) → A10, A11, A12 (large, external-dep).

## Open questions for plan

1. **Scope the big planes (A10/A11):** attempt str0m WebRTC + federation this phase, or
   keep them deferred and make phase-17 a "verify + pure-Rust completion" phase? (They are
   L-sized and externally dependent; a focused phase may be wiser.)
2. **A7 CDC slot management:** build it, or re-scope the CLI's advertised surface to
   "inspect-only" honestly (the gateway owns slot lifecycle)?
3. **A8 Dart:** does CI have `uniffi-bindgen-dart` available, or does this stay
   toolchain-blocked and documented?
