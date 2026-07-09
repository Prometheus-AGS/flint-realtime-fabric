# Plan — phase-24-sovereign-sfu-live-decode-and-flip

> Backend: **OpenSpec**. Generated 2026-07-08 from `assessment.md` + two operator decisions:
> **(1) bind `0.0.0.0` + advertised host-candidate IP + fixed UDP port** (config seam on
> `StrOmTransport`); **(2) the flip stays strictly conditional on a genuine `framesDecoded>0`** —
> if the bind-config/live-run work runs long, land what's done and **re-affirm the gate off**
> (confirmed on a second pass; the 8-phase discipline holds — no forced flip).
>
> Ordering: the real engineering (str0m bind-config) first, then the infra to exercise it, then
> the live proof, then the **conditional** flip. The flip is the *last* change and only lands on
> observed decoded media.

## Change list (ordered)

| # | Change id | What | Gate risk |
|---|-----------|------|-----------|
| c001 | `p24-c001-str0m-bind-config` | **GAP-1.** Add `MediaConfig { bind_addr, advertise_ip, udp_port }` + `StrOmTransport::with_config`; thread it through `create_session`→`negotiate` so the socket binds `0.0.0.0:<udp_port>` and the advertised host candidate uses `advertise_ip`. **Keep the loopback default** (`new()`) for tests. `main.rs` builds config from env (`MEDIA_ADVERTISE_IP`/`MEDIA_UDP_PORT`) for sovereign. Rust tests: config default = loopback; configured bind/advertise reflected in the host candidate. | **high** (the real work) |
| c002 | `p24-c002-compose-sovereign-media` | **GAP-2.** A `SFU_MODE=sovereign` compose path (service/override or profile) mapping the fixed UDP port (`<PORT>:<PORT>/udp`) + `MEDIA_ADVERTISE_IP` (host-reachable, e.g. `host.docker.internal`). Does **not** touch the production hosted `compose.yml` default. | med |
| c003 | `p24-c003-keto-view-seed` | **GAP-3.** A fixture/seed granting the harness subject `view` on the test room so the ADR-007 fail-closed join passes for a real run (a Keto tuple write via the seed path). No bypass of the check. | med |
| c004 | `p24-c004-dagger-decode-job` | **GAP-4.** A Dagger step that boots the sovereign stack + runs `media-decode.spec.ts` with Chromium fake-media (`SKIP_INTEGRATION=false`, `GATEWAY_URL`) and asserts `framesDecoded>0`; captures the run as evidence (or documents locally-run). | **high** (the proof) |
| c005 | `p24-c005-flip-or-reaffirm` | **G3.** If c004 observed a genuine `framesDecoded>0`: flip the `main.rs` sovereign branch (remove the warning) + SECURITY §6 (media → functional) + CHANGELOG. **Else: re-affirm gated** with the fresh failure detail; land c001–c004 as honest progress. Then **G4 carried** (LiveKit `realtime` cross-node now runnable; admin-ui OIDC still ADR-004+IdP-gated). PHASE-24-SIGNOFF. | med (honest branch) |

## Dependencies

- c002/c003 depend on c001 (the config seam + advertised candidate must exist).
- c004 depends on c001+c002+c003 (needs the bound socket, the UDP mapping, and the `view` grant).
- **c005 depends on c004's real outcome** — flip iff `framesDecoded>0` was genuinely observed;
  else re-affirm. **c005 must never flip on a relaxed/worked-around proof** (operator-confirmed).

## Notes

- **Dependency rule / one-port-per-adapter intact:** the bind-config lives in `frf-media-str0m`
  (its own concern); the gateway just passes env-derived config in. No authz/other-port bleed.
- **Loopback stays the default** so every existing str0m/gateway test is unaffected — the
  configurable path is additive.
- **Honest guard (operator-confirmed):** if GAP-1/live-run can't produce a clean `framesDecoded>0`
  this phase, split what's done, keep `SFU_MODE=sovereign` off, re-affirm gated. No forced flip.
- Each change passes the QA gate; **read the gate verdict AND the archive output** before archiving
  (phase-23 c002 + c005 lessons).
- First change to apply: **`p24-c001-str0m-bind-config`**.
