# Tasks — p18-c006

- [x] Bump str0m 0.7 → 0.21 (workspace Cargo.toml); resolve API changes
- [x] Adapter spike: on an SDP Offer envelope, Rtc::new + sdp_api().accept_offer, return Answer envelope
- [x] Trickle ICE: pipe IceCandidate envelopes into add_remote_candidate; emit SFU candidates back
- [x] UDP socket + poll_output/handle_input/timeout loop for one session; goal: DTLS connects
- [x] Deliverable: a passing round-trip test OR a documented findings note; SFU_MODE stays gated off unless media flows
- [x] clippy pedantic + unwrap_used clean; no file >500 lines
