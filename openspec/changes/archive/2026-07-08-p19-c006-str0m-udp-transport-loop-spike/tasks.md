# Tasks — p19-c006-str0m-udp-transport-loop-spike

- [x] 1. transport_spike.rs (new): `TransportLoop` (bound `UdpSocket` + `Rtc`), `bind()` adding the real local addr as a host candidate, `PollStep` + `poll_once()` (Transmit→send_to, Timeout→deadline, Event→surface), `feed_datagram()` (real `Input::Receive`)
- [x] 2. transport_spike.rs: test — bind + negotiate + `poll_once` yields a Transmit/Timeout step (loop turns); feed a datagram without panic
- [x] 3. lib.rs: register `pub mod transport_spike` + re-exports; SPIKE-FINDINGS.md: record what the transport-loop spike proves vs phase-20; SFU_MODE stays gated off
- [x] 4. Verify: cargo fmt + clippy (pedantic, unwrap_used) + tests green for frf-media-str0m
