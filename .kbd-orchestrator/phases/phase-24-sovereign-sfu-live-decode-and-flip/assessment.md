# Assessment — phase-24-sovereign-sfu-live-decode-and-flip

> Generated 2026-07-08. Gap report of the composed sovereign media plane against G1–G4 (bring up
> a live gateway, run the decode proof, flip the gate only on a real `framesDecoded > 0`).
> **The environment CAN host this** (docker running, Chromium installed) — but a real ICE media
> path across process boundaries needs str0m network-config work that does not exist yet.

## Environment reality (good news first)

- ✅ **Docker is running** (`docker info` OK) — the compose stack can come up here.
- ✅ **Chromium browser binaries installed** (`~/Library/Caches/ms-playwright/chromium-*`,
  `chromium_headless_shell-*`) + Playwright 1.61 — a live browser run is possible in this env.
- ✅ **Keto present in compose** (`oryd/keto:v0.12`) — the ADR-007 `view` grant can be seeded.
- ✅ **Harness authored** (phase-23): `media-decode.spec.ts` + `decode-probe.ts` +
  `webrtc-client.ts`, with the correct `framesDecoded` metric, integration-gated.

## The decisive gaps (why G2 cannot pass against today's build)

### GAP-1 — str0m binds loopback, no advertised public candidate  ·  **BLOCKER (code)**
`crates/frf-media-str0m/src/session.rs:139` binds `UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))` —
a **loopback, ephemeral** socket — and advertises a `127.0.0.1:<random>` host candidate
(`session.rs:147`). A browser in a different container/context **cannot reach `127.0.0.1`** on the
gateway, and the random port can't be mapped. `StrOmTransport::new()` takes **no config**
(`session.rs:69`), so there is no seam for a bind address, an advertised candidate IP, or a fixed
UDP port. **ICE cannot complete browser↔gateway across process boundaries.** This is the
load-bearing gap — a real decoded frame is impossible until it is fixed.

### GAP-2 — compose exposes no UDP media path, and runs hosted  ·  **BLOCKER (config)**
`compose.yml` gateway maps only `28080:8080` (TCP HTTP) and sets `SFU_MODE: "hosted"`. Even with
GAP-1 fixed, the container needs a **UDP port (range) mapping** and `SFU_MODE=sovereign` for a
browser to reach the media socket.

### GAP-3 — no Keto `view` seeding for the harness room  ·  **GAP (setup)**
ADR-007 makes room-join fail-closed on `check(subject,"view",room)`. A live run needs a Keto
relation tuple granting the harness subject `view` on the test room (or a documented
`DEV_NO_AUTH`-style harness tenant path) — else the join is correctly denied and no media flows.

### GAP-4 — no Dagger browser/e2e step  ·  **GAP (CI)**
`dagger/` has only `codegen.ts`; there is no step that boots the sovereign stack + runs Playwright
with Chromium fake-media. G2.2 (capture as CI evidence) needs one, or an honest locally-run record.

## Goal readiness

| Goal | Readiness |
|------|-----------|
| **G1** live sovereign gateway env | ◐ Partial — compose + Keto exist; needs `SFU_MODE=sovereign` + UDP mapping (GAP-2) + `view` seed (GAP-3). |
| **G2** live decoded-media run | ⛔ Blocked on **GAP-1** (str0m loopback bind — the real work) + GAP-2/3. Env is capable once these land. |
| **G3** flip `SFU_MODE=sovereign` | 🔒 Conditional on a real G2 pass. One-line branch + warning removal, only after `framesDecoded > 0` is observed. |
| **G4** LiveKit `realtime` / OIDC | ⏳ Carried; LiveKit cross-node now runnable (docker up); OIDC still needs ADR-004 + IdP. |

## Recommended plan order

1. **str0m bind/advertise config seam** (GAP-1) — `StrOmTransport::with_config` (or env-driven):
   bind address, advertised host-candidate IP, and a fixed/rangeable UDP port. This is the real
   engineering; keep the loopback default for tests, add the configurable path for deployment.
2. **compose sovereign media service** (GAP-2) — a `SFU_MODE=sovereign` gateway variant with a
   UDP port (range) mapped + the advertised-IP env from step 1.
3. **Keto `view` seed** (GAP-3) — a fixture tuple (or documented harness path) so the fail-closed
   join passes for the test subject.
4. **Dagger decode job** (GAP-4) — boot the stack, run `media-decode.spec.ts` with Chromium
   fake-media, assert `framesDecoded > 0`; capture evidence.
5. **Flip or re-affirm** (G3) — flip `main.rs` + SECURITY §6 + CHANGELOG **only** on a real G2
   pass; else re-affirm gated with the fresh failure detail.
6. **G4 carried proofs** as the env now supports them.

## Open questions for plan/analyze

1. **Advertised-IP strategy**: for a same-host browser↔container run, is `host.docker.internal` /
   the host LAN IP sufficient, or does the harness run the browser *inside* the compose network?
   (Simplest deterministic path: run Playwright against a gateway bound to a host-reachable IP.)
2. **UDP port strategy**: single fixed port (simplest for one session) vs a small mapped range
   (needed for concurrent sessions). Start with a fixed/small range for the 2-peer proof.
3. **If GAP-1 proves larger than one phase** (e.g. str0m needs STUN/host-candidate rework): split
   the bind-config work into its own change and keep the flip gated — do not force it.

## Honest bottom line

The environment is **capable** (docker + Chromium), which removes phase-23's "can't run here"
excuse. But a real decoded frame still needs **str0m network-config work (GAP-1)** first — the
loopback bind is the true blocker, not the harness. The flip stays conditional on G2 throughout.
