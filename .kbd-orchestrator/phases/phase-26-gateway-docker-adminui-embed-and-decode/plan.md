# Plan — phase-26-gateway-docker-adminui-embed-and-decode

> Backend: **OpenSpec**. Generated 2026-07-08 from `assessment.md`. The fix approach is
> unambiguous (multi-stage in-Docker Node build; host-COPY ruled out by `.dockerignore`) — no
> operator decision needed. A key simplification surfaced during planning: **admin-ui's `frf-wasm`
> dep is not required to build** — `vite.config.ts` substitutes a graceful stub when the wasm
> artifact is absent (it is not committed), so the Node stage needs **no Rust→wasm step**, just
> pnpm + vite. The flip stays strictly conditional on a genuine `framesDecoded > 0`.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p26-c001-dockerfile-adminui-embed` | Add a **Node 24 build stage** to the `Dockerfile`: copy the root pnpm workspace manifests (`package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`) + `admin-ui/` + the workspace member sources admin-ui imports (`sdks/entity-management`, `sdks/ts`), `corepack enable && pnpm install --frozen-lockfile`, `pnpm --dir admin-ui build` (vite uses the `frf-wasm` stub — no wasm build needed), then `COPY --from=<node> /build/admin-ui/dist ./admin-ui/dist` into the Rust builder **before** `cargo build`. Verify `docker compose -f compose.yml -f compose.sovereign.yml build gateway` succeeds. | **high** (the build fix; frozen-lockfile determinism in a clean image is the risk) |
| c002 | `p26-c002-live-decode-run` | **The proof — first run to reach the SFU.** Re-run `scripts/run-media-decode.sh` (authenticated path). The stack now boots; the harness authenticates, joins, and reaches ICE/DTLS/RTP. Observe `getStats().framesDecoded > 0`. Record the **actual** outcome in `docs/PHASE-26-DECODE-RESULT.md` — including, on failure, a concrete **media-path** diagnosis (advertise-IP / ICE / UDP reachability), now the most likely failure class. | **high** (the genuine, long-deferred media-path unknown) |
| c003 | `p26-c003-flip-or-reaffirm` | If c002 observed a genuine `framesDecoded > 0`: **flip** the `main.rs` sovereign branch (remove the warning) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-26-SIGNOFF.md`. **Else: re-affirm gated** with the fresh media-path detail. + **G4 carried**. | med (honest branch) |

## Dependencies

- c002 depends on c001 (the image must build + boot).
- **c003 depends on c002's real outcome** — flip iff `framesDecoded > 0` genuinely observed; else
  re-affirm. **Never flip on a relaxed/worked-around proof** (operator-confirmed, carried).

## Notes

- **No Rust→wasm stage needed** (planning simplification): `frf-wasm` is absent+gitignored and
  `vite.config.ts` falls back to a stub, so `vite build` succeeds without it. The admin UI's
  CRDT-wasm feature degrades in the image, but `dist` (what the gateway embeds) is produced — out of
  scope to fix the wasm feature here (non-goal: reworking the admin UI).
- **Workspace deps to copy** for the frozen install: `frf-sdk`, `entity-management` (`sdks/*`), and
  the `frf-wasm` package dir (stub-resolvable). Copy their sources; `.dockerignore` excludes only
  `node_modules`/`dist`, so sources are available.
- **compile-time embed:** the COPY of `dist` must precede `cargo build` (the release build embeds at
  compile time).
- **c002 is the real test** the last two phases never reached — ICE/DTLS/RTP over the
  `host.docker.internal` UDP candidate. A failure here is finally a *media-path* finding, valuable
  regardless; the gate holds.
- Each change passes the QA gate; **read the verdict AND the archive output** before archiving.
- First change to apply: **`p26-c001-dockerfile-adminui-embed`**.
