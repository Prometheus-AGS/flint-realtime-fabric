---
id: status
title: Project status
sidebar_label: Project status
---

What is proven, what is merely built, and what is deliberately off. This page
exists because the three are routinely conflated, and conflating them is how a
team convinces itself something works.

The vocabulary used throughout this site:

- **Proven** — a test was *executed* and observed to pass, or a behaviour was
  measured. Not "a test was written."
- **Built** — the code exists and compiles under the project's lint gates.
- **Gated off** — the code exists and is deliberately not enabled.

## Planes

| Plane | Status | Notes |
|---|---|---|
| Entity | **Live** | Postgres CDC → spine → per-event authorization → fan-out |
| Agent | **Live** | Authorized at subscribe time; revocation is not immediate |
| Media — signaling | **Live** | Both hosted and sovereign paths relay signaling |
| Media — sovereign SFU | **Proven, narrowly** | See below |
| Media — hosted (LiveKit) | **Live** | The supported v1 media path |
| Federation | **Built, off by default** | Requires explicit tenant and channel configuration |
| Relational replication (ADR-009) | **Built, gated off, uncertified** | Never run against a live Electric server |

## The sovereign SFU proof, precisely

The gate was flipped after a two-browser decode observed
`inbound-rtp.framesDecoded > 0` on the authenticated path, reproduced twice on
different container addresses.

**What that proves:** the relay decodes real media, on a local single-bridge
topology with both peers inside the container network and coturn available.

**What it does not prove:** multi-host operation, NAT traversal, or anything
about scale. The configuration is topology-sensitive — `MEDIA_ADVERTISE_IP` must
resolve to an address the peer can actually reach, and on a dual-stack bridge a
hostname can resolve IPv6-first and strand ICE in `new` with no error message.

`SFU_MODE` still defaults to `hosted`.

A related result is worth recording because it is easy to misread. An earlier CI
run showed `ice=connected` with roughly 1.8 MB received and
`framesDecoded=0` — a genuinely different failure on a Linux bridge, and one the
local proof does **not** address. Whether the keyframe-request fixes resolve it
is untested, because verifying it would require a CI test run, which this
project's policy forbids. It stays open as a known difference rather than a
claimed fix.

## Known documentation inconsistencies

Recorded here rather than left for a reader to trip over:

| Where | Issue |
|---|---|
| `docs/SECURITY.md` §6 | Still states the sovereign SFU gate is closed. It was flipped; use `docs/PHASE-36-LOCAL-DECODE-RESULT.md` as the current source. |
| `crates/frf-gateway/src/main.rs` | Two comments still say end-to-end media is unproven, contradicting the log line in the same file. |
| `ENVIRONMENT.md` vs `SECURITY.md` §6 | Federation is described as deferred in one and functional in the other. `SECURITY.md` is newer and more specific. |
| `frf-ports` shape facade | `AuthorizedShapeRequest` is documented as unconstructable outside the authorization path; in fact the guarantee is a disciplined seam, not a type-system one. |

## Decision records not yet accepted

ADR-004, ADR-005 and ADR-006 are marked **Proposed** despite 005 and 006 being
implemented. This is deliberate: *"existing implementation does not
retroactively record acceptance."*

## Testing policy

**CI never runs tests.** Not to verify a change, not to prove a gate, not once
as an exception. CI runs build, lint, typecheck, format and packaging gates.

All testing is local full-integration testing against a locally-composed stack.
A task whose only path to completion is a CI run is unsatisfiable and must be
rewritten around a local run.

This is why some items above are marked built rather than proven: proving them
requires local integration runs that have not happened yet, and this
documentation will not promote them on the strength of a test that exists but
has never executed.

## What would most improve this picture

1. **Execute the authored-but-unrun tests.** Several dozen tests across the
   related repositories are written, type-checked, and have never run.
2. **A multi-host media proof.** The local proof is real and narrow. The next
   honest claim requires a different topology.
3. **Run the shape facade against a live Electric server.** Until then, ADR-009
   is a design, not a feature.
