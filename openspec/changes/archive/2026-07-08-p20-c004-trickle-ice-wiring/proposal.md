# p20-c004 — trickle ICE wiring

## Why

The async session engine (c003) has the inbound candidate command path
(`add_remote_candidate` → driver → `Candidate::from_sdp_string`) but `local_signals` is a
stub, so the **outbound** half of trickle ICE (the session's own candidates + connection
state) is not relayed. This change wires trickle ICE both directions and unit-tests the
envelope↔candidate mapping.

## Design finding (honest scope)

str0m's `Event` enum has **no per-local-candidate event** — in its sans-I/O model local
candidates are advertised in the SDP answer (the host candidate seeded at bind), and
server-reflexive (srflx) candidates come from feeding STUN. So "emit local candidates
outbound" is **not a live str0m event stream**: the outbound `local_signals` stream carries
the session's **host candidate** (captured at bind) + **connection-state changes** (which
str0m *does* emit as events). Full srflx gathering + connectivity checks need a STUN server
and a real peer — **browser/infra-gated**, documented, not faked.

## What Changes

1. **`Cargo.toml`:** promote `chrono` from dev-only to a normal dependency (outbound
   envelopes need a timestamp in library code).
2. **`ice.rs` (new):** envelope↔candidate helpers — `candidate_string_from_envelope`
   (inbound: extract the candidate from an `IceCandidate` envelope payload),
   `candidate_envelope` / `state_envelope` (outbound: build `IceCandidate` / state
   `SignalEnvelope`s). Unit-tested deterministically (no socket).
3. **`session.rs`:** `SessionHandle` gains a `local_signals` broadcast; at `create_session`
   the host candidate is captured and broadcast as an `IceCandidate` envelope; the driver
   broadcasts a state envelope on each `ConnectionState` change; `local_signals` returns a
   real `BroadcastStream`. The inbound path uses `candidate_string_from_envelope`.
4. **Test:** the inbound mapping (envelope → candidate string) round-trips; `local_signals`
   yields the host-candidate envelope after `create_session`.

## Non-goals

- srflx candidate gathering / STUN / connectivity checks (browser/infra-gated).
- RTP forwarding (phase-21); enabling `SFU_MODE=sovereign` (still gated off).

## Impact

- Affected: `crates/frf-media-str0m/src/{ice.rs,session.rs,lib.rs}`, `Cargo.toml`.
- Trickle ICE is wired both directions with the mapping unit-tested; live connectivity is
  honestly browser/STUN-gated.
