//! Unit tests for `session.rs` — extracted (p24-c001) to keep `session.rs` under the
//! 500-line limit. Included via `#[path = "session_tests.rs"] mod tests;` so `super::*`
//! resolves to the `session` module exactly as if inline.

use std::net::SocketAddr;

use super::*;
use crate::driver::RECV_BUF;
use str0m::media::{Direction, MediaKind};
use str0m::net::{Protocol, Receive};
use str0m::{Input, Output};

/// A real remote SDP offer from a second `Rtc`, so the round-trip uses str0m's own
/// 0.21 offer format.
fn make_offer() -> String {
    let mut remote = Rtc::builder().build(Instant::now());
    let addr = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0));
    remote.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
    let mut change = remote.sdp_api();
    change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
    let (offer, _pending) = change.apply().expect("offer produced");
    offer.to_sdp_string()
}

#[tokio::test]
async fn create_session_starts_a_driver_and_returns_an_answer() {
    let transport = StrOmTransport::new();
    let sid = SessionId::new();
    let tid = TenantId::new();

    let answer = transport
        .create_session(sid, tid, &make_offer())
        .await
        .expect("create_session should negotiate + spawn");
    assert!(answer.starts_with("v=0"), "answer should be valid SDP");
    assert!(
        answer.contains("m=audio"),
        "answer should carry the offered media"
    );

    // The session is registered and its state starts at New (the async driver turns the
    // loop in the background — we only assert the engine is live here, no browser).
    let state = transport.connection_state(sid).await.expect("state");
    assert!(matches!(
        state,
        ConnectionState::New | ConnectionState::Connecting
    ));

    // Teardown stops the driver and deregisters.
    transport.remove_session(sid, tid).await.expect("remove");
    assert!(transport.connection_state(sid).await.is_err());
}

#[tokio::test]
async fn local_mdns_candidate_is_skipped_not_fatal() {
    // p29-c002 (phase-28 B1): Chrome sends host candidates as mDNS `<uuid>.local` names str0m
    // cannot parse. The demux command handler must SKIP that one candidate and keep the session
    // alive — never error the whole session. Sent through the public `add_remote_candidate`, so
    // it exercises the real command → demux `apply_command` path.
    let transport = StrOmTransport::new();
    let sid = SessionId::new();
    let tid = TenantId::new();
    transport
        .create_session(sid, tid, &make_offer())
        .await
        .expect("create_session");

    // A realistic browser mDNS host candidate — no IP, only a `.local` name.
    let mdns = crate::ice::candidate_envelope(
        tid,
        sid,
        "",
        "candidate:1 1 udp 2113937151 abcd1234-0000-0000-0000-abcdef012345.local 54321 typ host",
    );
    // The call itself succeeds (the bad candidate is skipped inside the demux, not rejected here).
    transport
        .add_remote_candidate(sid, mdns)
        .await
        .expect("a .local candidate is accepted-then-skipped, not an error");

    // The session is still alive after the skipped candidate — its state is still queryable and
    // it has NOT been reaped.
    assert!(
        transport.connection_state(sid).await.is_ok(),
        "session must survive an unparseable .local candidate"
    );

    transport.remove_session(sid, tid).await.expect("remove");
}

#[tokio::test]
async fn unknown_session_ops_report_cleanly() {
    let transport = StrOmTransport::new();
    let sid = SessionId::new();
    assert!(transport.connection_state(sid).await.is_err());
    // remove of an unknown session is a no-op success (idempotent teardown).
    transport
        .remove_session(sid, TenantId::new())
        .await
        .expect("remove of unknown session is a no-op");
}

#[tokio::test]
async fn local_signals_yields_the_host_candidate_envelope() {
    let transport = StrOmTransport::new();
    let sid = SessionId::new();
    let tid = TenantId::new();
    transport
        .create_session(sid, tid, &make_offer())
        .await
        .expect("create_session");

    // The outbound trickle-ICE stream's first item is the session's local host
    // candidate — proven regardless of subscription timing (it is prepended).
    let mut stream = transport.local_signals(sid).await.expect("local_signals");
    let first = stream.next().await.expect("an envelope").expect("no err");
    assert_eq!(first.kind, frf_domain::SignalKind::IceCandidate);
    assert_eq!(first.from_session, sid);
    let cand = crate::ice::candidate_string_from_envelope(&first);
    assert!(
        cand.is_some_and(|c| c.contains("candidate") && c.contains("host")),
        "first local signal should be the host candidate"
    );

    transport.remove_session(sid, tid).await.expect("remove");
}

#[tokio::test]
async fn configured_advertise_ip_appears_in_the_host_candidate() {
    // p24-c001 / p28-c002: with an explicit advertise host (IP or hostname), the SDP host candidate
    // must advertise *that* IP (not loopback/0.0.0.0) — what lets a browser reach the socket.
    let transport = StrOmTransport::with_config(crate::config::MediaConfig {
        bind_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        advertise_host: Some("203.0.113.7".to_owned()),
        udp_port: 0,
    });
    let sid = SessionId::new();
    let tid = TenantId::new();
    transport
        .create_session(sid, tid, &make_offer())
        .await
        .expect("create_session");

    let mut stream = transport.local_signals(sid).await.expect("local_signals");
    let first = stream.next().await.expect("an envelope").expect("no err");
    let cand = crate::ice::candidate_string_from_envelope(&first).expect("candidate");
    assert!(
        cand.contains("203.0.113.7"),
        "advertised candidate must carry the configured IP, got: {cand}"
    );

    transport.remove_session(sid, tid).await.expect("remove");
}

#[tokio::test]
async fn wait_for_connected_times_out_without_a_peer() {
    // A single answerer session has no peer to complete ICE/DTLS with, so it never
    // reaches Connected. This proves the crypto provider installs without panic and the
    // wait mechanic returns the (non-Connected) current state on timeout — not a hang,
    // not a fake Connected.
    let transport = StrOmTransport::new();
    let sid = SessionId::new();
    let tid = TenantId::new();
    transport
        .create_session(sid, tid, &make_offer())
        .await
        .expect("create_session");

    let state = transport
        .wait_for_connected(sid, std::time::Duration::from_millis(150))
        .await
        .expect("known session");
    assert_ne!(
        state,
        ConnectionState::Connected,
        "no peer ⇒ must not reach Connected"
    );

    transport.remove_session(sid, tid).await.expect("remove");
}

#[tokio::test]
async fn join_room_co_locates_peers_for_forwarding() {
    // Two sessions joined to the same room become forwarding peers (each forwards to the
    // other); removing one deregisters it so the survivor has no peer. This proves the
    // StrOmTransport ↔ RoomRouter create/join/remove wiring (the router's own fan-out is
    // unit-tested in room.rs).
    let transport = StrOmTransport::new();
    let tid = TenantId::new();
    let a = SessionId::new();
    let b = SessionId::new();
    transport
        .create_session(a, tid, &make_offer())
        .await
        .expect("a");
    transport
        .create_session(b, tid, &make_offer())
        .await
        .expect("b");

    transport.join_room(a, tid, "room-x");
    transport.join_room(b, tid, "room-x");
    assert_eq!(transport.router().peer_count(a), 1, "A forwards to B");
    assert_eq!(transport.router().peer_count(b), 1, "B forwards to A");

    transport.remove_session(b, tid).await.expect("remove b");
    assert_eq!(
        transport.router().peer_count(a),
        0,
        "after B leaves, A has no forwarding peer"
    );

    transport.remove_session(a, tid).await.expect("remove a");
}

#[tokio::test]
async fn two_sessions_share_one_fixed_media_port() {
    // p29-c001 / ADR-008 (phase-28 B2): the transport binds ONE shared UDP socket on a fixed
    // port and demuxes to per-session `Rtc` by `accepts()`. Two sessions on a fixed port must
    // both negotiate with no `Address already in use` — under the old per-session-bind model the
    // second `create_session` failed `EADDRINUSE`. Bind an ephemeral fixed port first (port 0
    // resolves to a concrete free port on the shared socket) and create two sessions against it.
    let transport = StrOmTransport::with_config(crate::config::MediaConfig {
        bind_addr: std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        advertise_host: None,
        // A fixed non-zero port is the case that collided under the old per-session bind. Use a
        // high port unlikely to be taken in test; the shared socket binds it exactly once.
        udp_port: 41_247,
    });
    let tid = TenantId::new();
    let a = SessionId::new();
    let b = SessionId::new();

    transport
        .create_session(a, tid, &make_offer())
        .await
        .expect("first session binds the shared fixed port");
    transport
        .create_session(b, tid, &make_offer())
        .await
        .expect("second session REUSES the shared socket — no EADDRINUSE");

    // Both are live and advertise the same shared bound port in their host candidate.
    let mut sa = transport.local_signals(a).await.expect("a signals");
    let mut sb = transport.local_signals(b).await.expect("b signals");
    let ca =
        crate::ice::candidate_string_from_envelope(&sa.next().await.expect("a env").expect("a ok"))
            .expect("a candidate");
    let cb =
        crate::ice::candidate_string_from_envelope(&sb.next().await.expect("b env").expect("b ok"))
            .expect("b candidate");
    assert!(
        ca.contains("41247"),
        "A advertises the shared port, got: {ca}"
    );
    assert!(
        cb.contains("41247"),
        "B advertises the shared port, got: {cb}"
    );

    transport.remove_session(a, tid).await.expect("remove a");
    transport.remove_session(b, tid).await.expect("remove b");
}

/// Build an `Rtc` bound to a real loopback UDP socket, with its socket address seeded as
/// a host candidate. Returns the `Rtc` and its blocking socket.
fn peer_rtc() -> (Rtc, std::net::UdpSocket) {
    let socket =
        std::net::UdpSocket::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("bind loopback udp");
    socket.set_nonblocking(true).expect("nonblocking");
    let addr = socket.local_addr().expect("local addr");
    let mut rtc = Rtc::builder().build(Instant::now());
    rtc.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
    (rtc, socket)
}

/// Drive one `Rtc` a single step: drain its outputs (send Transmits over `socket`), then
/// pump any pending inbound datagram into `handle_input`. Returns the next timeout.
fn pump(rtc: &mut Rtc, socket: &std::net::UdpSocket, buf: &mut [u8]) {
    loop {
        match rtc.poll_output().expect("poll_output") {
            Output::Timeout(_) => break,
            Output::Transmit(t) => {
                let _ = socket.send_to(&t.contents, t.destination);
            }
            Output::Event(_) => {}
        }
    }
    // Non-blocking receive: feed whatever is queued.
    while let Ok((n, source)) = socket.recv_from(buf) {
        let local = socket.local_addr().expect("local addr");
        if let Ok(r) = Receive::new(Protocol::Udp, source, local, &buf[..n]) {
            let _ = rtc.handle_input(Input::Receive(Instant::now(), r));
        }
    }
    // Advance the clock so DTLS/ICE timers fire.
    let _ = rtc.handle_input(Input::Timeout(Instant::now()));
}

/// Full DTLS-connected proof: an offerer and an answerer complete a real ICE connectivity
/// check + DTLS handshake to `Connected` in-process over loopback (no browser). The
/// offerer role is a **test harness** — the `MediaTransport` port stays answerer-only (the
/// SFU always answers a browser's offer).
#[test]
fn two_peers_reach_dtls_connected() {
    install_crypto_provider();

    let (mut l, l_sock) = peer_rtc();
    let (mut r, r_sock) = peer_rtc();
    let l_addr = l_sock.local_addr().expect("l addr");
    let r_addr = r_sock.local_addr().expect("r addr");

    // Offerer (l) creates an offer; answerer (r) accepts it; l applies the answer.
    let mut change = l.sdp_api();
    change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
    let (offer, pending) = change.apply().expect("offer");
    let answer = r.sdp_api().accept_offer(offer).expect("answer");
    l.sdp_api()
        .accept_answer(pending, answer)
        .expect("apply answer");

    // Give each peer the other's socket address as a remote host candidate so ICE has a
    // pair to check.
    l.add_remote_candidate(Candidate::host(r_addr, "udp").expect("r cand"));
    r.add_remote_candidate(Candidate::host(l_addr, "udp").expect("l cand"));

    // Drive both peers until connected, bounded so a failure can't hang.
    let mut buf = vec![0_u8; RECV_BUF];
    let mut connected = false;
    for _ in 0..2000 {
        pump(&mut l, &l_sock, &mut buf);
        pump(&mut r, &r_sock, &mut buf);
        if l.is_connected() && r.is_connected() {
            connected = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(
        connected,
        "two peers must complete ICE + DTLS to Connected over loopback"
    );
}
