//! str0m live-UDP **transport-loop spike** (p19-c006).
//!
//! The negotiation spike ([`crate::rtc_spike`]) proved str0m 0.21 can turn an SDP offer
//! into an answer. This spike proves the next load-bearing unknown from `SPIKE-FINDINGS.md`:
//! the **sans-I/O UDP event loop** — bind a real socket, drive `Rtc::poll_output` and
//! `Rtc::handle_input` over it, and actually transmit outbound datagrams. That loop is where
//! WebRTC interop breaks and is hard to test without a browser; here we prove the *mechanics*
//! turn over a real socket, deterministically, with no browser.
//!
//! Deliberately **out of scope** (deferred to the phase-20 sovereign SFU media loop): full
//! trickle ICE (candidate gathering + connectivity checks), the DTLS/SRTP handshake to a
//! real peer, RTP `MediaData` forwarding, and per-room fan-out. This spike does not move
//! media, and `SFU_MODE=sovereign` stays gated off.

use std::net::{SocketAddr, UdpSocket};
use std::time::Instant;

use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Event, Input, Output, Rtc, RtcError};

/// Errors surfaced by the transport-loop spike.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TransportError {
    /// Binding the UDP socket or reading its local address failed.
    #[error("udp socket error: {0}")]
    Socket(#[from] std::io::Error),
    /// A host ICE candidate could not be constructed from the socket address.
    #[error("candidate error: {0}")]
    Candidate(String),
    /// str0m rejected an input or failed while producing output.
    #[error("str0m error: {0}")]
    Rtc(#[from] RtcError),
    /// An inbound datagram could not be parsed into a str0m `Receive`.
    #[error("malformed inbound datagram: {0}")]
    Datagram(String),
}

/// What one turn of the event loop did — the observable result of [`TransportLoop::poll_once`].
#[derive(Debug)]
#[non_exhaustive]
pub enum PollStep {
    /// str0m produced an outbound datagram, which was sent over the socket.
    Transmitted {
        /// Destination the datagram was sent to.
        to: SocketAddr,
        /// Number of bytes written.
        len: usize,
    },
    /// str0m wants to be polled again no later than this instant (no output ready now).
    Timeout(Instant),
    /// str0m surfaced a connection/media event. Boxed because str0m's `Event` is large
    /// (str0m itself allows `large_enum_variant` on its `Output` for the same reason).
    Event(Box<Event>),
}

/// A minimal sans-I/O transport loop: a real bound `UdpSocket` paired with an `Rtc`.
///
/// This is a spike, not the production SFU: it owns exactly one `Rtc` and drives the
/// str0m event loop over a real socket to prove the loop turns. A production SFU keeps a
/// per-session `Rtc` + socket + task and forwards RTP between peers (phase-20).
pub struct TransportLoop {
    socket: UdpSocket,
    local_addr: SocketAddr,
    rtc: Rtc,
}

impl TransportLoop {
    /// Bind a UDP socket on loopback and seed the `Rtc` with the socket's **real** local
    /// address as a host candidate (unlike the negotiation spike's port-0 placeholder).
    ///
    /// # Errors
    ///
    /// [`TransportError::Socket`] if the bind or local-address read fails;
    /// [`TransportError::Candidate`] if str0m rejects the host candidate.
    pub fn bind() -> Result<Self, TransportError> {
        let socket = UdpSocket::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
        socket.set_nonblocking(true)?;
        let local_addr = socket.local_addr()?;

        let mut rtc = Rtc::builder().build(Instant::now());
        let candidate = Candidate::host(local_addr, "udp")
            .map_err(|e| TransportError::Candidate(e.to_string()))?;
        rtc.add_local_candidate(candidate);

        Ok(Self {
            socket,
            local_addr,
            rtc,
        })
    }

    /// The socket's bound local address (host candidate address).
    #[must_use]
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Accept a remote SDP offer, returning the answer SDP. Mirrors the negotiation spike
    /// but on the `Rtc` that owns the live socket, so a subsequent `poll_once` drives real
    /// I/O.
    ///
    /// # Errors
    ///
    /// [`TransportError::Rtc`] if str0m rejects the offer.
    pub fn accept_offer(
        &mut self,
        offer: str0m::change::SdpOffer,
    ) -> Result<String, TransportError> {
        let answer = self.rtc.sdp_api().accept_offer(offer)?;
        Ok(answer.to_sdp_string())
    }

    /// Turn the event loop once: poll str0m for output and act on it. On `Transmit` the
    /// datagram is actually written to the socket; on `Timeout` the deadline is returned;
    /// on `Event` it is surfaced. This is the core of the sans-I/O loop.
    ///
    /// # Errors
    ///
    /// [`TransportError::Rtc`] if `poll_output` errors; [`TransportError::Socket`] if the
    /// outbound `send_to` fails.
    pub fn poll_once(&mut self) -> Result<PollStep, TransportError> {
        match self.rtc.poll_output()? {
            Output::Transmit(t) => {
                let len = self.socket.send_to(&t.contents, t.destination)?;
                Ok(PollStep::Transmitted {
                    to: t.destination,
                    len,
                })
            }
            Output::Timeout(deadline) => Ok(PollStep::Timeout(deadline)),
            Output::Event(event) => Ok(PollStep::Event(Box::new(event))),
        }
    }

    /// Feed one inbound datagram into the `Rtc` as an `Input::Receive`, with the socket's
    /// address as the destination. This is the receive half of the sans-I/O loop.
    ///
    /// # Errors
    ///
    /// [`TransportError::Datagram`] if the bytes are not a datagram str0m accepts;
    /// [`TransportError::Rtc`] if `handle_input` rejects the receive.
    pub fn feed_datagram(&mut self, buf: &[u8], from: SocketAddr) -> Result<(), TransportError> {
        let receive = Receive::new(Protocol::Udp, from, self.local_addr, buf)
            .map_err(|e| TransportError::Datagram(e.to_string()))?;
        self.rtc
            .handle_input(Input::Receive(Instant::now(), receive))?;
        Ok(())
    }

    /// Advance str0m's clock with an `Input::Timeout` (the no-network branch of the loop).
    ///
    /// # Errors
    ///
    /// [`TransportError::Rtc`] if str0m rejects the timeout input.
    pub fn drive_timeout(&mut self, now: Instant) -> Result<(), TransportError> {
        self.rtc.handle_input(Input::Timeout(now))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use str0m::media::{Direction, MediaKind};

    /// Build a real remote SDP offer from a second `Rtc` (str0m's own format), so the
    /// round-trip is not driven by a hand-written SDP that could drift from 0.21.
    fn make_offer() -> str0m::change::SdpOffer {
        let mut remote = Rtc::builder().build(Instant::now());
        let addr = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0));
        remote.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
        let mut change = remote.sdp_api();
        change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
        let (offer, _pending) = change.apply().expect("offer produced");
        offer
    }

    #[test]
    fn bind_uses_the_real_socket_address_as_candidate() {
        let loop_ = TransportLoop::bind().expect("bind should succeed");
        // A real bound socket has a non-zero port — unlike the negotiation spike's port-0
        // placeholder. This is the whole point: the loop advertises a real local address.
        assert_ne!(
            loop_.local_addr().port(),
            0,
            "bound socket must have a real port"
        );
        assert!(loop_.local_addr().ip().is_loopback());
    }

    #[test]
    fn the_event_loop_turns_over_a_real_socket() {
        // The load-bearing assertion: after negotiating, the sans-I/O loop turns and
        // produces a Transmit or Timeout step over a real socket — no browser, no panic.
        let mut loop_ = TransportLoop::bind().expect("bind");
        let _answer = loop_.accept_offer(make_offer()).expect("negotiate");

        // Drive the clock so str0m has work to do, then poll a few times. A freshly
        // negotiated Rtc emits STUN/ICE transmits and/or a timeout — either proves the
        // loop turns. We only need one non-error step within a bounded number of polls.
        loop_
            .drive_timeout(Instant::now())
            .expect("timeout input accepted");

        let mut turned = false;
        for _ in 0..8 {
            match loop_.poll_once().expect("poll_once must not error") {
                PollStep::Transmitted { len, .. } => {
                    assert!(len > 0, "a transmit must carry bytes");
                    turned = true;
                    break;
                }
                PollStep::Timeout(_) => {
                    // The loop reached a stable "poll me later" state — it turned cleanly.
                    turned = true;
                    break;
                }
                PollStep::Event(_) => {
                    // A connection/media event is also the loop turning; keep polling for a
                    // terminal Transmit/Timeout, but note progress.
                    turned = true;
                }
            }
        }
        assert!(turned, "the event loop must produce at least one step");
    }

    #[test]
    fn feeding_a_non_stun_datagram_does_not_panic() {
        // A stray/non-STUN datagram from the network must be handled without taking the
        // loop down — either accepted or cleanly rejected, never a panic.
        let mut loop_ = TransportLoop::bind().expect("bind");
        let _answer = loop_.accept_offer(make_offer()).expect("negotiate");
        let from = SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 54321));
        // str0m may reject this as a malformed/unknown datagram — that's fine; the contract
        // is "no panic". Both Ok and Err are acceptable outcomes.
        let _ = loop_.feed_datagram(&[0x00, 0x01, 0x02, 0x03], from);
    }
}
