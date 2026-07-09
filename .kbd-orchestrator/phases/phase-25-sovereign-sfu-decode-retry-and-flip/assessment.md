# Assessment — phase-25-sovereign-sfu-decode-retry-and-flip

> Generated 2026-07-08. Narrow gap report: the media plane is built; this phase makes the decode
> run reach the browser, observe `framesDecoded > 0`, and flip only on a real pass. The phase-24
> blocker is precisely understood, so the gaps are tight.

## Phase-24 blocker — root cause confirmed (G1)

The runner's no-`E2E_JWT` fallback chained `compose.override.example.yml` as a third `-f`. That
dev override:
- Redefines `gateway.depends_on` as a **list** (`[iggy-server]`), which **replaces** the base's
  map-form `depends_on` (keto/postgres/iggy with health conditions), and
- Moves `keto`, `postgres`, `surrealdb`, `flint-gate` to `profiles: ["full"]` (disabled unless
  `--profile full`).

So chained without `--profile full`, the base still references keto/postgres → **`service
"gateway" depends on undefined service "keto"`** → invalid project. The **sovereign-only merge
(`compose.yml + compose.sovereign.yml`) is valid** — only this fallback breaks.

## G1 — the two viable fixes (both concrete)

### Option A — flint-gate `E2E_JWT` (preferred; exercises c003)
- flint-gate is in compose (host `14457:4457`) with a **`mint_jwt` route** (`deploy/flint-gate/config.yaml:57`),
  HS256 signed by `FLINT_GATE_JWT_SECRET`, injecting `sub: dev-integration-user`,
  `tenant_id: …0001`. The gateway verifies it via `GATEWAY_JWKS_URL` (flint-gate JWKS).
- **Fix:** bring flint-gate up (it is in the base stack), mint a JWT via its route, pass it as
  `E2E_JWT` to the runner. The harness then joins the room with a **real authenticated subject**,
  which the c003 authz checks against a seeded `(sub=dev-integration-user, view, room)` grant.
- This is the faithful path — it proves the authenticated ADR-007 flow end to end.

### Option B — minimal clean no-auth override
- A **purpose-built** `compose.sovereign-dev.yml` that adds only `DEV_NO_AUTH=true`,
  `CARGO_FEATURES=dev-endpoints` (Dockerfile supports the build arg — `Dockerfile:17`), and the
  `SFU_MODE`/`MEDIA_*` env — **without** touching `depends_on` or moving services to profiles.
- Caveat: `DEV_NO_AUTH` requires a `dev-endpoints` rebuild and **bypasses the c003 authz** (the WS
  path resolves tenant from the `tenant` query param, no subject). Proves media *flow*, not the
  authenticated authz. Weaker; use only if Option A stalls.

## Gaps (against G1–G4)

| Goal | Readiness / gap |
|------|-----------------|
| **G1** runner live path | ◐ Two concrete fixes (A: flint-gate JWT mint step in the runner; B: minimal no-auth override). Option A needs a small runner change to mint + pass `E2E_JWT`; seed the grant for `sub=dev-integration-user`. |
| **G2** real decoded frame | ⛔ Blocked on G1. Once the stack boots + the harness authenticates, run `media-decode.spec.ts` and observe `framesDecoded>0`. **Unknown until run** whether ICE/DTLS/RTP completes over the `host.docker.internal` UDP path — the next real risk after G1. |
| **G3** flip | 🔒 Conditional on a real G2 pass. |
| **G4** LiveKit/OIDC | ⏳ Carried. |

## Open questions for plan/analyze

1. **A vs B**: recommend **Option A** (flint-gate JWT) — it exercises the c003 authenticated-subject
   authz, so the proof covers the real path, not a bypass. B is the fallback if minting proves
   fiddly.
2. **ICE reachability**: `MEDIA_ADVERTISE_IP=host.docker.internal` advertises the host to the
   browser; on macOS Docker Desktop the browser runs on the host and reaches the mapped UDP port
   there. Confirm the fake-media Chromium actually completes ICE to that candidate — this is the
   G2 risk that only a run reveals. If it fails, diagnose media-path vs. advertise-IP and record it.
3. **Cold build cost**: the sovereign gateway rebuild (esp. with `dev-endpoints` for Option B) is
   several minutes in Docker — run the live attempt in the background.

## Recommendation

Plan order: **G1 runner fix (Option A: flint-gate JWT mint + seed for the minted sub)** → **G2 live
run → observe `framesDecoded>0`** → **G3 flip-or-reaffirm** (conditional on G2) → **G4 carried**.
Keep the flip conditional on a real frame throughout; if G2 fails again, record the concrete blocker
(now more likely a genuine media-path issue than harness) and hold the gate.
