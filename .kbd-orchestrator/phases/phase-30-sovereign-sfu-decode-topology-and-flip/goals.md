# Goals — phase-30-sovereign-sfu-decode-topology-and-flip

> Seeded from: phase-29 (reflection `summaryForNext`). Phase-29 cleared BOTH phase-28 blockers and
> proved them live: **B2** — one shared demuxing `UdpSocket` (ADR-008, `Rtc::accepts()`); two
> sessions on one port, no `EADDRINUSE`. **B1** — coturn STUN + harness `iceServers` + tested
> `.local` skip; browser `remoteCandidates` 0→1, ICE `new`→`disconnected`. But the decode still
> yields **no frame**: neither session reaches `Connected`, zero `MediaData`. Root cause (evidence,
> `docs/PHASE-29-DECODE-RESULT.md`): the browser (on the host) and the SFU (in a container) never
> share a routable candidate pair — the STUN `srflx` reflects the Docker **bridge** view while the
> gateway advertises host **loopback** `127.0.0.1`. This is the last classic browser↔SFU reality.

This phase fixes the **same-host candidate topology** so a real pair completes to `Connected`, RTP
fans out, and a receiver decodes a frame — and **only then** flips `SFU_MODE=sovereign`. The SFU
engine is already proven correct (negotiate → shared-socket demux → `.local` skip → two-peer room);
the remaining work is **environmental (harness/compose), not engine code**.

**Discipline (carried 16→29):** `SFU_MODE=sovereign` flips on **only** once decoded media provably
flows to a real receiver against a live gateway; else it stays gated off with fresh rationale. No
"healthy but does nothing." Update SECURITY §6 + CHANGELOG as it lands; re-run the gate suite. Seed
the openspec change dir at the START of every apply; read the QA verdict AND the archive output.
Update progress.json to N/N before any command mentioning the next stage (pipeline-enforce guards on
the pre-update snapshot — phase-29 lesson).

---

## G1 — Fix the same-host candidate topology so a pair completes to Connected

Phase-29 evidence: `ice=disconnected`, no `Connected`, zero `MediaData` — the one pair that forms is
host-loopback (gateway `127.0.0.1`) vs. bridge-srflx (browser via coturn) and can't sustain
connectivity.

- **G1.1 (primary):** run the **browser inside the Docker network** — a Playwright/Chromium service
  on the compose network — so its host/srflx candidates and the gateway's `0.0.0.0:40000` are on one
  network and a pair completes. Most faithful + hermetic.
- **G1.2 (alternatives, if G1.1 is impractical):** advertise a **bridge-reachable
  `MEDIA_ADVERTISE_IP`** (the host-gateway IP — already supported by `resolve_advertised_ip`) so the
  browser's srflx can pair with it; **or** give the gateway **host networking** in the sovereign
  override so `127.0.0.1` is literally shared. Pick one; all are harness/compose, not engine.
- **G1.3** Keep the `MediaConfig`/`RoomRouter`/demux/bridge contracts intact (no engine change unless
  the evidence demands one — ADR if so). File-size ≤500; no library `unwrap`/`expect`.

**Exit:** the decode run reaches `ice=connected` and at least one session logs `state=Connected` +
inbound `MediaData` at the gateway (fan-out begins), or the concrete blocker is diagnosed + recorded.

## G2 — Observe a real decoded frame + flip

- **G2.1** Re-run `scripts/run-media-decode.sh`; observe `getStats().framesDecoded > 0`. Capture
  `docs/PHASE-30-DECODE-RESULT.md` (browser assertion + gateway str0m logs).
- **G2.2** **Only on a genuine pass:** flip the `main.rs` sovereign branch (remove the gate-off
  warning → live path) + SECURITY §6 (media → functional) + CHANGELOG + `docs/PHASE-30-SIGNOFF.md`.
  Else re-affirm gated with fresh rationale. Conditional on G2.1, always (operator-confirmed, carried).

**Exit:** `SFU_MODE=sovereign` moves real decoded media, or stays gated with fresh rationale.

## G3 — Carried live proofs (from phase-19…29)

- **G3.1** LiveKit cross-node inbound (`realtime` feature). **G3.2** admin-ui OIDC (ADR-004 + IdP).

**Exit:** each passes against real infra, or is re-affirmed integration-gated.

---

## Phase completion criteria

- Each goal functions end-to-end (proven) or is re-affirmed deferred with fresh rationale in
  SECURITY §6 + CHANGELOG.
- Release gate green; each change passes the QA gate (**read the verdict AND the archive output**;
  **seed the openspec change dir at the start of every apply**).
- `SFU_MODE=sovereign` is enabled **only** if G2 observes a real `framesDecoded > 0`.

## Non-goals

- Re-opening ADR-001/003/004/005/006/007/008 without a new finding.
- TURN relay, simulcast, or net-new media features beyond what the decode proof + flip require (STUN
  srflx already lands in c002; TURN only if the topology fix genuinely needs a relay).

## Starting point (from phase-29)

- Shared demuxing socket (ADR-008), STUN srflx + `.local` skip, coturn in compose, harness `STUN_URL`.
- The SFU engine is proven correct; the gap is purely same-host ICE candidate topology.
- Blocker to clear: **browser↔SFU routable candidate pair on one host** (G1).
