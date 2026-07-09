# Tasks — p29-c002-stun-srflx-path

- [x] 1. Add a coturn STUN service to compose.sovereign.yml (3478/udp) for the decode run.
- [x] 2. Configure the decode probe RTCPeerConnection with iceServers (STUN_URL, default stun:127.0.0.1:3478) so Chrome gathers an srflx candidate.
- [x] 3. Explicit unit test: an unparseable `.local` mDNS remote candidate is skipped (never fatal) by the demux command handler.
