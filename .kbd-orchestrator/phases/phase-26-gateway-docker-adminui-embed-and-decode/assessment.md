# Assessment — phase-26-gateway-docker-adminui-embed-and-decode

> Generated 2026-07-08. Narrow gap report: one Dockerfile defect blocks the gateway image; fixing
> it lets the proof boot for the first time and finally test the media path. Every finding below is
> concrete.

## The defect — fully characterized (G1)

- `frf-gateway` embeds the admin UI at **compile time**: `admin_ui.rs:16-18` —
  `#[derive(RustEmbed)] #[folder = "../../admin-ui/dist"] struct AdminUiAssets;`. `rust-embed = "8"`
  with **no `debug-embed` feature**, and the image is a **release** build → the folder is embedded
  at compile time, so `admin-ui/dist` **must exist before `cargo build --release`**.
- The current `Dockerfile` copies only `Cargo.toml`/`Cargo.lock`, `crates/`, `proto/` — **never**
  `admin-ui/dist`. So inside the image the folder is absent → `RustEmbed` fails (`folder does not
  exist`, `E0599 no get for AdminUiAssets`) → the gateway lib won't compile → the stack never boots.
- **Host builds pass** only because a prior `vite build` left `admin-ui/dist/` on disk; the gap is
  Docker-context-only.

## Fix constraints (these shape the change)

1. **Host-build + COPY is ruled out.** `admin-ui/dist` is in `.dockerignore` (lines 8-12,
   `admin-ui/dist/`), so a host-built `dist` is not in the build context. The clean fix is a
   **multi-stage in-Docker Node build** (`COPY --from=<node-stage>`).
2. **admin-ui is a pnpm workspace member.** Root `pnpm-workspace.yaml` lists `admin-ui` (+ `sdks/*`).
   The Node stage must copy the **root** `package.json` + `pnpm-lock.yaml` + `pnpm-workspace.yaml`
   (and the workspace members it needs) — not just `admin-ui/` — for a frozen-lockfile install.
3. **Toolchain:** `admin-ui` needs **Node ≥24**, pnpm (via corepack), builds with
   `tsc --noEmit && vite build` (vite 7, react 19). `esbuild` is in `allowedBuilds`.
4. **Ordering:** the Node stage must produce `dist` and the Rust builder must `COPY --from=<node>
   /…/admin-ui/dist ./admin-ui/dist` **before** the `cargo build` line (Dockerfile:19-24).

## Gaps (against G1–G4)

| Goal | Readiness / gap |
|------|-----------------|
| **G1** Dockerfile embed fix | ◐ Well-scoped: add a Node build stage + COPY `dist` into the Rust context before `cargo build`. The `.dockerignore` on `admin-ui/dist` is fine (we build in-image); confirm the Node stage's `COPY` sources aren't over-excluded by `.dockerignore` (it excludes `node_modules`, `dist` — both fine, they're produced in-stage). |
| **G2** live decode run | ⛔ Blocked on G1. Once the image builds + boots, this is the **first run to reach the SFU** — the genuine ICE/DTLS/RTP-over-UDP test. Real risk lives here now. |
| **G3** flip | 🔒 Conditional on a real G2 pass. |
| **G4** LiveKit/OIDC | ⏳ Carried. |

## Open questions for plan/analyze

1. **Node install scope**: copy the whole repo's pnpm workspace manifests + `admin-ui` (minimal), or
   the full context? Minimal (root `package.json`/`pnpm-lock.yaml`/`pnpm-workspace.yaml` + `admin-ui`
   + any workspace deps admin-ui imports) keeps the stage cache-friendly. Confirm admin-ui has no
   cross-workspace runtime deps on `sdks/*` that must also be copied.
2. **Build determinism**: `pnpm install --frozen-lockfile` requires the lockfile to be current;
   confirm it installs cleanly in a clean Node 24 image (a stale lockfile would fail the stage).
3. **The media-path unknown (G2)**: unchanged from phase-25 — whether Chromium completes ICE/DTLS/RTP
   to the `host.docker.internal` UDP candidate. Only the (now-reachable) run reveals it; if it fails,
   diagnose advertise-IP / ICE / UDP reachability concretely and hold the gate.

## Recommendation

Plan order: **G1 Dockerfile multi-stage Node build** (build `admin-ui/dist` in a Node 24 stage,
COPY into the Rust context before `cargo build`; verify `docker compose … build gateway` succeeds)
→ **G2 re-run the authenticated proof → observe `framesDecoded > 0`** (the first real media-path
test) → **G3 flip-or-reaffirm** (conditional on G2) → **G4 carried**. This is the tightest phase in
the sequence — one well-understood build fix unlocks the long-deferred decode test.
