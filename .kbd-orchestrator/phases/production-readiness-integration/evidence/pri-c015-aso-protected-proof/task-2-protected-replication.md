# Task 2 — current-source protected replication

Result: **Passed**

Observed at: `2026-09-17T15:08:26.395449+00:00`

## Selected topology

The c003 release contract selects the `shape-only` browser topology: Gate,
Fabric's HTTP shape facade, Electric, PostgreSQL and browser PGlite through the
pinned PEM packages. Native ASO is outside this release slice, so no native
topology is claimed.

## Current-source result

The aggregate campaign exercised the protected Gate/FRF shape feed through the
transactional SQL materializer and into the PEM graph. It passed with:

- ASO HEAD `d95242542060d1e19a6efe9143f57c015d6162f4` and selected-source aggregate
  `9c380d4144256d412299c487a445446201c028b43470d51ca872d40b5a65aa1a`;
- Fabric HEAD `ae26f45d8afa037677004cd118b0571b8b552131` and selected-source aggregate
  `02af23d67a1dd708658c628ae8b222575450b538836ccff4235bbf9d9f9cd624`;
- Gate HEAD `77914a2ad2e3cd57f12fd9131fa8cdddf625f68e` and selected-source aggregate
  `b3d9398ff7a2b1bada6ce3a49860553d9da9b48fc07b630c18ea24a95e5bfc5b`.

All three selected-source aggregates and repository revisions were identical
before and after the run.

The production graph-provider import closure contained 34 files at aggregate
`337cdc470558f989acc0149c7098826226fa17ecf001fff4b3d321f349c9fe76`.
It wired `createFrfShapeTransport` to `startReplicaRuntime` and contained zero
generic entity-watch writer symbols. The protected clinical replica therefore
has one authoritative writer.

## Functional proof

- The focused ASO suite passed 99/99 tests across transport, runtime,
  ownership, checkpoint writing, projection and catalog boundaries.
- The Fabric protected-stream revocation suite passed 4/4 tests.
- The mounted browser campaign materialized the 16,203-row representative
  dataset with SQL/graph parity and all seven shape checkpoints.
- Protected composition passed fresh-grant authorization, handle ownership,
  practice and projection denial, per-request revalidation, HTTP 409 expiry,
  and authorized cold refetch.
- The client segment could reach Gate and received 401 without a session. It
  could not resolve or reach FRF, Electric, or the operator loopback diagnostic
  directly. The diagnostic remained reachable only from operator loopback.
- Run-owned rows, PGlite storage, services, callback server, and two synthetic
  identities were cleaned successfully.

## Source-bound artifacts

| Artifact | SHA-256 |
|---|---|
| `campaign.json` | `c2c388d317cc152a88985b3b18c1d7744fac2ffa54efce710fed87a85158352c` |
| `task-4-mounted-materializer.json` | `cdf892aedefa82f6426026b0d5c214fc9c172c97e5010008d32cde36b5df56db` |
| `task-4-mounted-browser-memory.json` | `77513863cf455c2373768489e4137cbe47869a36e2b8ae762966f5fbdb1967a6` |
| `protected-composition.json` | `135556833989edf32456d92e08a38e8ada70119b22f25a9f5339baa3792fe957` |
| `protected-topology.json` | `03fdee99349993fd220fd99fc2073e9f272dafa6a671b3d607bb6f57d5aa1ef0` |

Earlier failed attempts remain alongside the authoritative passing artifacts.
They record the callback-port, memory-sample, expiry-status, and missing
Compose-environment corrections instead of being used as passing evidence.
