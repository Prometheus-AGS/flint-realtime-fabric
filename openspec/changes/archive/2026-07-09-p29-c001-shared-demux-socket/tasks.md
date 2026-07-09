# Tasks — p29-c001-shared-demux-socket

- [x] 1. ADR-008 (docs/decisions/adr-008-shared-media-socket.md): shared-socket ownership decision, refines ADR-005/006.
- [x] 2. StrOmTransport owns one shared UdpSocket; negotiate stops per-session bind; single demux loop (demux.rs) routes by Rtc::accepts(); files ≤500.
- [x] 3. Multi-session in-process test: two sessions on one fixed port (no EADDRINUSE) + accepts-demux routes A's datagram to A's Rtc; existing str0m tests green.
