# Assessment — phase-18-media-federation-and-auth-flow

> Generated: 2026-07-07 · Backend: OpenSpec
> Method: 3 parallel code-grounded audits (admin-ui auth / str0m SFU / federation+Dart) +
> deterministic dep/version checks. Every claim cites file:line.
> Baseline: all four plane crates compile clean; phases 16–17 released the hosted shape.

This phase's four goals each have **existing scaffold code**, so the assessment grounds
each against its real current state — and surfaces **three findings that materially change
the plan** vs. the seed goals.

---

## Headline — the seed goals' ordering assumption is wrong

The reflection seeded G1 (admin-ui login) as "cheapest, do first, no new backend
dependency." **That assumption does not hold.** The audit found G1 is actually
**blocked on a backend/IdP decision**, while G2 (str0m) hides a **signaling routing bug
that breaks even the non-WebRTC path**. The real cheapest wins are small correctness
fixes (federation channel-ID guard, str0m routing bug, Dart doc drift), not G1. Re-order
accordingly.

---

## G1 — admin-ui interactive login → **BLOCKED on a backend decision (not cheap)**

**What exists:** clean, well-abstracted plumbing. `authStore.ts` (Zustand) + `useAuth.ts`
+ the `authInterceptor` in `infrastructure/gateway.ts:20-26` (sets `Authorization:
Bearer`). `authService.ts`'s own comment says `login()` is designed to *become* the
redirect handler. Gateway-side JWT verification is done (G1.3 already satisfied).

**What's missing / blocking:**
- **Current "login" is a paste-a-JWT gate** — `LoginGate.tsx` renders a password input;
  `authService.ts` stores the token in `localStorage` (`frf.accessToken`). No redirect,
  no PKCE, no refresh, no expiry handling.
- **flint-gate exposes NO interactive login endpoint.** Its `/oauth/token` supports only
  RFC 8693 **token-exchange** and **client_credentials** — **no `authorization_code`, no
  `/authorize`, no `/callback`, no `/.well-known/openid-configuration`.** PKCE code exists
  only in the MCP DCR path, not a browser flow.
- **Kratos (the actual IdP per the stack table) is NOT deployed** — not in `compose.yml`
  or `deploy/`. flint-gate dev config runs `dev_passthrough` (anonymous).
- admin-ui has **no OIDC library** and a **hand-rolled hash router** (`App.tsx`) that
  can't cleanly handle a path-based `/callback`. No OIDC `VITE_*` env vars exist.

**→ Load-bearing open question for the plan:** *what does the UI authenticate against?*
Options: (a) stand up Kratos/Hydra, front it with flint-gate token-exchange, do OIDC
auth-code+PKCE in the UI; (b) build an `authorization_code` endpoint in flint-gate
(separate repo); (c) keep paste-a-JWT and re-scope G1 as "improve the dev token flow"
honestly. **G1 cannot start until this is decided.**

---

## G2 — str0m sovereign SFU → **NOT STARTED + a live routing bug**

**What exists:** `frf-media-str0m` is **channel-based signaling routing only** — a
`DashMap<(TenantId,SessionId), mpsc::Sender>`. **No `str0m::Rtc`, no SDP/ICE/DTLS/RTP.**
The `str0m` crate is declared (`0.7`) but **never imported** (only in comments/span
names). `StrOmError::Ice`/`Dtls` variants exist but are never constructed.

**NEW CRITICAL FINDING — routing bug (blocks even signaling):** `send_signal` keys on
`(tenant_id, from_session)` — the **sender's own** session — and **never reads
`to_session` or `room_id`** (`sfu.rs:58`). So a signal from peer A is delivered back to
**A's own** receive channel, never to peer B. The unit test passes only because it
sends/subscribes as the *same* session, masking the bug. **This matches the phase-16
audit's flagged detail — it was never fixed.** Even pure signaling relay is broken today.

**Other findings:**
- **str0m `0.7` is pinned but latest is `0.21.0`** — a 14-minor-version gap. Any real
  build needs a dependency bump + API-currency pass first.
- **`sfu_mode` is forced to `Hosted`** in the gRPC mapping regardless of config
  (`signal_service.rs:157,173`) — a wire-level inconsistency.
- **`Default for GatewayConfig` uses `Sovereign`** while `from_env` defaults `Hosted`
  (`config.rs:122` vs `275-282`) — inconsistent (only affects tests).

**The port fits SDP/ICE (they ride in the opaque `payload`) so a signaling round-trip
needs no port change; real media (RTP) does not fit** — pure server-side relay can stay
in the 3-method trait, but a media surface/event loop would need extending.

**Sizing:** spike (one `Rtc` round-trip, adapter-only) ≈ 1–2 days *after* the routing bug
is fixed; full build (per-session `Rtc` + UDP/ICE/DTLS event loop + RTP fan-out) ≈ 2–4
weeks. **Biggest risk:** the sans-I/O UDP/ICE/DTLS transport loop — hard to test without a
real browser, and it forces replacing the channel-only architecture with per-session
async transport tasks.

---

## G3 — Federation → **mixed: 3 work, 3 stub/broken, + a config gap**

| Direction | State | Evidence | To complete |
|-----------|-------|----------|-------------|
| Matrix **outbound** | ✅ works | real HTTP PUT `client.rs:86-106` | — |
| Matrix **inbound** | ❌ stub | `stream::empty()` `client.rs:83` | `/sync` long-poll over reqwest — **can unblock now** (doesn't need the Tuwunel crate, despite the `BLOCKED_ON_TUWUNEL` note) |
| ATProto **inbound** | ✅ works | live Jetstream WS + reconnect `jetstream.rs:28-86` | — |
| ATProto **outbound** | ❌ returns `Err` | `lib.rs:60-62` | PDS auth/session + `com.atproto.repo.createRecord` |
| LiveKit **outbound** | ✅ works (cross-node) | `send_data` REST fan-out `adapter.rs:92-101` | — |
| LiveKit **inbound** | ❌ in-process only | local broadcast `adapter.rs:117-133` | LiveKit realtime SDK data-channel client |

- **Gating is correct:** `FEDERATION_ENABLED` off by default, bridges only built when on,
  warns on misconfig.
- **NEW finding — config gap:** `validate()` requires `FEDERATION_TENANT_ID` when enabled,
  but **`FEDERATION_CHANNEL_ID` has no such guard** — `main.rs:273-275` silently falls
  back to a per-boot random `ChannelId::new()` even in production. Federated events would
  land under a random channel. **Cheap XS fix.**

---

## G4 — Dart async-transport bindings → **still broken; no upstream fix; docs stale**

- The two known errors persist and one got **worse**: `connect` now type-declares
  `Future<FrfFfiClient>` but its body returns non-Future **and** throws
  `UnsupportedError('runtime invocation ... not implemented yet (connect)')`
  (`frf.dart:1970-71,1254-55`). `subscribe` still passes `EventCallback` where a u64
  handle is expected (`frf.dart:2008,1608`).
- **`uniffi-bindgen-dart` 0.1.3 is the latest published version** — **no upstream fix**
  for async-constructor / foreign-callback codegen. So G4.1 (regenerate with a newer
  version) is a dead end; **G4.2 (hand-written Dart shim over the sync FFI) is the only
  viable path.**
- The generated dir is analyzer-excluded (masks the errors — hidden, documented).
- **NEW finding — doc drift:** `sdks/dart/pubspec.yaml:2` still credits
  "flutter_rust_bridge 2.11.1" and lists it as a dependency, contradicting ADR-003. XS fix.

---

## Gap summary → phase-18 change candidates

| ID | Gap | Goal | Severity | Size |
|----|-----|------|----------|------|
| **B1** | str0m signaling **routing bug** (`from_session`, ignores `to_session`/`room_id`) | G2 pre-req | **HIGH** (blocks all media) | S |
| B2 | Federation `FEDERATION_CHANNEL_ID` validate() guard missing | G3 | MEDIUM | XS |
| B3 | Dart doc drift: pubspec credits flutter_rust_bridge; GENERATED.md `connect` shape stale | G4 | LOW | XS |
| B4 | `sfu_mode` forced Hosted on the wire; `Default` vs `from_env` mode mismatch | G2 | LOW | XS |
| A1 | **Decide the G1 auth backend** (Kratos/Hydra vs flint-gate auth-code vs re-scope) | G1 | — | decision |
| A2 | admin-ui OIDC auth-code+PKCE flow (dep + `/callback` route + refresh) — *after A1* | G1 | — | L |
| A3 | str0m spike: one real `Rtc` round-trip (bump to 0.21, adapter-only) | G2 | — | M |
| A4 | str0m full: per-session `Rtc` + UDP/ICE/DTLS loop + RTP fan-out | G2 | — | XL |
| A5 | Matrix inbound `/sync` long-poll (reqwest; no Tuwunel dep) | G3 | — | M |
| A6 | ATProto outbound PDS write (`createRecord`) | G3 | — | M |
| A7 | LiveKit cross-node inbound relay (realtime data-channel client) | G3 | — | L |
| A8 | Dart async-transport hand-written shim over sync FFI | G4 | — | M |

**Recommended ordering for `/kbd-plan`:** cheap correctness first — **B1 (str0m routing),
B2 (channel guard), B3 (Dart docs), B4 (sfu_mode)** — then the **G1 auth-backend decision
(A1)** before any G1 frontend work; then the plane builds by cost: A3 str0m spike →
A5/A6 (Matrix inbound, ATProto outbound — self-contained) → A8 Dart shim → A2 G1 frontend
→ A7 LiveKit inbound → A4 str0m full (largest; consider its own sub-phase).

## Open questions for plan

1. **G1 auth backend (blocking):** stand up Kratos/Hydra, add an auth-code endpoint to
   flint-gate, or honestly re-scope G1 to "improve the dev token flow"? Nothing in G1 can
   proceed until this is answered.
2. **str0m scope:** is the full SFU (A4, ~2–4 wks, XL) in phase-18, or does phase-18 do
   **B1 + the A3 spike only** and split the full media build into a dedicated phase-19?
3. **str0m 0.7 → 0.21 bump:** acceptable to bump now (14 minor versions; API churn), or
   pin-and-defer?
4. **Federation completeness:** finish all three broken directions (A5/A6/A7) this phase,
   or the two cheap self-contained ones (Matrix inbound, ATProto outbound) and defer
   LiveKit cross-node?
