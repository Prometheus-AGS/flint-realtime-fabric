# Reflection — phase-36-sovereign-sfu-ice-linux-fix

> Reflected 2026-09-08. **3 of 4 changes complete; c002 deferred and superseded.**
>
> The phase achieved its goal: `SFU_MODE=sovereign` is ON, flipped on a genuine
> `framesDecoded > 0`. The gate held shut since phase 16 opened on evidence, not
> on patience running out.

## Goal achievement

| Goal | Verdict | Evidence |
|---|---|---|
| G1 — fix gateway-log capture | **MET, but not as diagnosed** | The capture was never broken; runs died before reaching it. 25,638 bytes on the first run that got there. |
| G2 — read the candidate log, diagnose the ICE stall | **MET** | Advertised candidate was `fd07:b51a:cc66:d002::6` — an IPv6 ULA the peer could not pair with. |
| G3 — fix ICE pairing | **MET** | Runner pins the container's resolved IPv4. ICE connects; decode passes. |
| G4 — flip on genuine `framesDecoded > 0` | **MET** | `decode_exit=0`, `1 passed`, reproduced twice on different container IPs. |

## The finding that matters

**Every defect this phase fixed was in the harness or the local path. None was in
the SFU.** Three of the four made a working media path look broken:

| Defect | Symptom it produced |
|---|---|
| `FLINT_GATE_JWT_SECRET` unexported | compose aborted before any container started |
| `GATEWAY_URL` used `localhost` (IPv6-first) | a healthy gateway reported "never became healthy" |
| no `node_modules` on the host | aborted at the Playwright exec |
| `MEDIA_ADVERTISE_IP` resolving IPv6-first | ICE stranded in `new`, `bytes=0` |

**Two of the four were the same root cause** — a name resolving IPv6-before-IPv4 —
in two different places. That single class of bug was the most expensive thing in
the phase, and it is invisible in every log: the gateway says it is listening, the
port says it is published, and the request still fails.

Phases 28–36 accumulated evidence that read as media-path trouble. At least the
local portion of it was the harness the whole time.

## Corrections I made to my own work

Recorded because a reflection that only lists successes teaches nothing:

- **I reported the c001 log capture as broken, twice.** It was not. The `cp` at
  `:184` was correctly ordered before the EXIT trap all along; the runs simply
  never reached that line. I wrote "third failure of this capture path; it should
  stop being treated as fixed" into `tasks.md` — that sentence was wrong, and it
  was wrong in the confident register that is hardest to walk back.
- **I recorded the runtime as Docker Desktop.** It is OrbStack. That mattered:
  the IPv6 forwarding behaviour is OrbStack's.
- **I classified the failed run as (d) HARNESS and declared the phase question
  "exactly as open as it was".** True when written, but I stopped one diagnostic
  step short — the cause was fixable and the decode passed 20 minutes later. The
  classification discipline was right; the stopping point was premature.
- **My first spec delta silently dropped an existing scenario.** `openspec
  validate --strict` caught it. A MODIFIED block replaces the whole requirement,
  and I had also rewritten the requirement's prose, which would have deleted the
  log-capture clause.

## What the taxonomy could not express

T4 offered three outcomes: (a) decoded, (b) ICE connected but no frames, (c) ICE
never connects. The first run fit none of them — it aborted before Playwright, so
there was no probe reading at all. Forcing it into (c) would have manufactured an
ICE finding out of a harness abort.

I added **(d) HARNESS** rather than bend the result. That turned out to be the
right call for a reason I did not anticipate: (d) is *weaker* than (b) or (c), and
naming it that way kept the phase honest for the twenty minutes before the real
answer arrived. The spec delta now encodes this — an aborted run must not be
reported as an ICE or media-path result.

## What is proven, and what is not

**Proven:** the sovereign SFU relays media a real browser decodes, on a local
single-bridge topology with both peers inside the container network. PLI fires and
`room=` is populated at runtime, confirming the ADR-009 child's fixes.

**Not proven, and not claimed:**
- Multi-host, NAT-traversal, or scale behaviour. The proof is single-bridge.
- That the PLI produces a keyframe *at the sender*. Generation and routing are
  observed; the sender-side response is not.
- That the CI failure (run 29112243615: `ice=connected`, ~1.8 MB,
  `framesDecoded=0`) is fixed. That is a **different** failure on a Linux bridge
  where none of these four defects apply. Verifying it needs a CI run, which
  policy forbids. It stays open as a known difference, not a claimed fix.

The flip's log line carries the topology caveat so an operator meets it at
runtime rather than in this document.

## Technical debt

| Debt | Where | Why accepted |
|---|---|---|
| `p36-c002` deferred with 2 tasks permanently blocked | openspec/changes | Both require a CI run. Superseded by c004; not archived, because archiving reads as complete. |
| CI decode path still unverified after the PLI/room fixes | `.github/workflows/decode-proof.yml` | Policy forbids the run that would verify it. |
| `MEDIA_ADVERTISE_IP` default still the ambiguous hostname | `compose.sovereign.yml` | Correct on the CI Linux bridge; changing the default would alter CI behaviour for a local-only problem. |
| Sender-side keyframe response unobserved | frf-media-str0m | Needs a probe the harness does not have. |

## Lessons

1. **Two IPv6-before-IPv4 resolutions cost more than any code defect this phase.**
   On a dual-stack bridge, a hostname is ambiguous and the resolver decides. Pin
   the family anywhere an address must be *reachable by a specific peer*.
2. **"The harness is broken" and "the system is broken" produce identical logs.**
   Every abort here looked like a media-path failure from the outside. Confirm the
   measurement ran before interpreting what it measured.
3. **A false claim stated confidently is worse than one stated tentatively.** I
   wrote that the capture path "should stop being treated as fixed" into a
   durable artifact. It was correct in that moment and wrong about the cause.
4. **Exit codes through pipes lie — again.** `docker compose up` returned
   `UP_EXIT=0` while printing a fatal interpolation error. Fourth instance this
   session. Read the output, not just the status.
5. **Classify before interpreting, and say so when nothing fits.** Adding (d)
   rather than forcing (c) is the only reason the interim record was not wrong.

## Recommended focus for the next phase

1. **Do not chase the CI decode failure.** It cannot be verified under the local-
   testing-only policy. Either reproduce that topology locally (a Linux bridge
   without the OrbStack IPv6 behaviour) or leave it recorded as a known difference.
2. **Verify the phase-37 child's 85 authored-but-unexecuted tests.** That backlog
   is unchanged by this phase and is still the largest unproven surface in the repo.
3. **Sweep for other IPv6-ambiguous names** anywhere the system advertises or polls
   an address that another party must reach.
4. **Fix `build-review-packet.sh`** to accept extra repo roots — carried from the
   phase-37 child, where every cross-repo artifact BLOCKs for a tooling reason.
