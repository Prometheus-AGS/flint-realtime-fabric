# str0m Sovereign SFU — Spike Findings (p18-c006)

> Deliverable of the phase-18 str0m spike. Goal: prove one real WebRTC round-trip on
> str0m 0.21 before committing to the full SFU build (deferred to a later phase).

## What the spike proved ✅

1. **str0m 0.7 → 0.21 bump is clean.** The 14-minor-version jump resolved and compiles;
   the crate previously *declared but never used* str0m, so nothing regressed. New
   transitive deps (`str0m-aws-lc-rs`, `str0m-proto`, `x509-*`) come in but build fine.
2. **The negotiation state machine works end-to-end.** `crates/frf-media-str0m/src/rtc_spike.rs`
   implements `negotiate_answer(offer_sdp) -> answer_sdp`:
   - `Rtc::builder().build(Instant::now())` + a host `Candidate` + `sdp_api().accept_offer(offer)`
     yields a well-formed `SdpAnswer`.
   - Proven by `accept_offer_produces_an_answer` — a real offer generated from a *second*
     `Rtc` (`sdp_api().add_media(Audio, SendRecv).apply()`) negotiates to an SDP answer
     that carries the offered `m=audio` line. `invalid_offer_is_rejected` covers the error
     path. Both tests pass with **no browser and no UDP** — pure in-process negotiation.

This is the load-bearing unknown resolved: the str0m API is correctly wired and the SDP
offer→answer half of a sovereign SFU functions.

## Transport-loop spike (p19-c006) ✅

The second load-bearing unknown — the **sans-I/O UDP event loop** — is now de-risked in
`crates/frf-media-str0m/src/transport_spike.rs` (`TransportLoop`):

- **A real bound `UdpSocket` is used.** `bind()` binds a loopback UDP socket and advertises
  its **actual local address** as a host `Candidate` (not the negotiation spike's port-0
  placeholder). `bind_uses_the_real_socket_address_as_candidate` asserts a non-zero port.
- **The event loop turns over the socket.** `poll_once()` drains `rtc.poll_output()`:
  `Output::Transmit` → real `socket.send_to`, `Output::Timeout` → deadline,
  `Output::Event` → surfaced. `the_event_loop_turns_over_a_real_socket` negotiates a real
  offer, drives an `Input::Timeout`, and asserts the loop reaches a `Transmit`/`Timeout`
  step — proving the sans-I/O mechanics turn end-to-end, **no browser**.
- **Inbound receive is wired.** `feed_datagram()` builds a real
  `Input::Receive(Protocol::Udp, ..)` and calls `rtc.handle_input`;
  `feeding_a_non_stun_datagram_does_not_panic` proves a stray datagram is handled (accepted
  or cleanly rejected) without taking the loop down.

What this proves: the `poll_output`/`handle_input`/`send_to` loop over a real socket is
correctly wired on str0m 0.21. It does **not** run a full ICE connectivity check, DTLS
handshake, or move media — that is the phase-20 scope below.

## Phase-20 progress (p20-c003 → c005)

The transport loop is now a real per-session async engine implementing the `MediaTransport`
port (`session.rs`, `StrOmTransport`):

- **Per-session async architecture.** ✅ *p20-c003.* A per-session tokio task owns an `Rtc`
  + a tokio `UdpSocket`, driving str0m's canonical loop (`select!` over the `poll_output`
  timeout deadline, `recv_from`, and a command channel). Replaces the blocking spike shape.
- **Trickle ICE.** ✅ *p20-c004* (plumbing). Inbound `IceCandidate` envelopes →
  `add_remote_candidate`; the session's local **host** candidate + connection-state changes
  are relayed outbound via `local_signals`. Finding: str0m has **no per-local-candidate
  event** — candidates ride the SDP answer / come from STUN, so srflx gathering +
  connectivity checks remain **browser/STUN-gated**.
- **DTLS crypto + connected milestone.** ✅ *p20-c005.* The engine installs the crypto
  provider (`from_feature_flags().install_process_default()`, once) so DTLS can key, and
  exposes `wait_for_connected(session, timeout)`. **Honest boundary:** reaching
  `Event::Connected` needs a peer in the **offerer** role (the port only exposes the
  answerer `create_session(offer)→answer` path) + real connectivity — so the two-peer
  DTLS-connected proof is `#[ignore]` integration-gated (`two_peers_reach_dtls_connected`),
  not faked. The `wait_for_connected` mechanic + crypto install are proven (a peerless
  session times out without reaching Connected, no panic).

## What the full build still needs (deferred to phase-21) ⏳

- **RTP forwarding.** str0m handles DTLS internally, but the SFU must relay
  `Event::MediaData` between session peers (`rtc.writer(mid).write(..)`), keyed by
  mid/track, with per-room fan-out topology, keyframe (PLI) handling, and renegotiation.
  **This is the phase-21 thrust** — reaching `Connected` (phase-20) is not media flowing.
- **Offerer/peer role + live connectivity proof.** The two-peer DTLS-connected +
  media-exchange proof needs an offerer role and a real peer/browser — integration-gated.
- **srflx/STUN gathering + connectivity checks** for real (non-loopback) ICE.

## Interop risk (updated by p19-c006)

The **loop mechanics** (bind → `poll_output`/`send_to` → `handle_input`) are now proven
over a real socket (p19-c006). The remaining risk narrows to **live browser interop** — a
real ICE connectivity check + DTLS handshake + SRTP to an actual browser peer — which still
cannot be proven without a browser and is where WebRTC interop breaks in practice. The two
spikes together de-risk the str0m API and the sans-I/O loop; they do not de-risk end-to-end
browser interop, which phase-20 proves with a real client.

## Status of the gate

`SFU_MODE=sovereign` **remains gated off** (default `hosted`, boots with a warning). No
media flows through the sovereign path; the spike is a proof of negotiation, not a live
SFU. Do not advertise sovereign as shipped until the full build lands.
