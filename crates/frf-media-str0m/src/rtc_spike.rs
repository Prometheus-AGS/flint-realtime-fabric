//! str0m WebRTC round-trip **spike** (p18-c006).
//!
//! This proves the str0m 0.21 `Rtc` state machine can accept an SDP offer and produce a
//! valid answer — the negotiation half of a real sovereign SFU. It is deliberately
//! adapter-only and does **not** run a live UDP/ICE/DTLS media loop; that
//! (`poll_output`/`handle_input` over real sockets, RTP forwarding, per-room fan-out) is
//! the remaining full-build work, deferred to a dedicated phase. `SFU_MODE=sovereign`
//! stays gated off until media actually flows.
//!
//! What this spike establishes:
//! - the str0m 0.7 → 0.21 bump compiles and the negotiation API is wired correctly;
//! - `Rtc::builder().build()` + a host `Candidate` + `sdp_api().accept_offer(offer)`
//!   yields a well-formed `SdpAnswer` for a real remote offer.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Instant;

use str0m::change::{SdpAnswer, SdpOffer};
use str0m::{Candidate, Rtc, RtcError};

/// Errors surfaced by the negotiation spike.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SpikeError {
    /// The offer SDP could not be parsed.
    #[error("invalid SDP offer: {0}")]
    InvalidOffer(String),
    /// str0m rejected the offer or failed to build the RTC state machine.
    #[error("str0m negotiation error: {0}")]
    Negotiation(#[from] RtcError),
    /// A host ICE candidate could not be constructed.
    #[error("candidate error: {0}")]
    Candidate(String),
}

/// Build a fresh `Rtc` with a single host candidate on a bindable local address.
///
/// The candidate address is a placeholder for the spike — a real SFU binds a UDP socket
/// and advertises its actual local/reflexive candidates. Here we only need a syntactically
/// valid host candidate so `accept_offer` can produce an answer.
fn build_rtc() -> Result<Rtc, SpikeError> {
    let mut rtc = Rtc::builder().build(Instant::now());
    // Port 0 is fine for negotiation; a real SFU would bind a socket and use its addr.
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
    let candidate =
        Candidate::host(addr, "udp").map_err(|e| SpikeError::Candidate(e.to_string()))?;
    rtc.add_local_candidate(candidate);
    Ok(rtc)
}

/// Accept a remote SDP offer and return the SFU's SDP answer.
///
/// This is the negotiation round-trip: parse the offer, drive it through a str0m `Rtc`
/// state machine, and serialize the resulting answer back to SDP text. The `Rtc` is
/// dropped afterwards — the spike proves negotiation, not a persistent media session.
///
/// # Errors
///
/// [`SpikeError::InvalidOffer`] if the SDP does not parse; [`SpikeError::Negotiation`] if
/// str0m rejects the offer.
pub fn negotiate_answer(offer_sdp: &str) -> Result<String, SpikeError> {
    let offer = SdpOffer::from_sdp_string(offer_sdp)
        .map_err(|e| SpikeError::InvalidOffer(e.to_string()))?;
    let mut rtc = build_rtc()?;
    let answer: SdpAnswer = rtc.sdp_api().accept_offer(offer)?;
    Ok(answer.to_sdp_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use str0m::media::{Direction, MediaKind};

    /// Generate a real remote SDP offer from a second `Rtc`, so the round-trip is driven
    /// by str0m's own offer format (not a hand-written SDP that might drift from 0.21).
    fn make_offer() -> String {
        let mut remote = Rtc::builder().build(Instant::now());
        let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        remote.add_local_candidate(Candidate::host(addr, "udp").expect("host candidate"));
        let mut change = remote.sdp_api();
        change.add_media(MediaKind::Audio, Direction::SendRecv, None, None, None);
        let (offer, _pending) = change.apply().expect("offer produced");
        offer.to_sdp_string()
    }

    #[test]
    fn accept_offer_produces_an_answer() {
        // The spike's core assertion: a real offer negotiates to a well-formed answer.
        let offer_sdp = make_offer();
        let answer_sdp = negotiate_answer(&offer_sdp).expect("negotiation succeeds");
        assert!(
            answer_sdp.starts_with("v=0"),
            "answer should be a valid SDP block, got: {answer_sdp}"
        );
        // An answer must carry the media section that was offered.
        assert!(
            answer_sdp.contains("m=audio"),
            "answer should include the audio m-line, got: {answer_sdp}"
        );
    }

    #[test]
    fn invalid_offer_is_rejected() {
        let err = negotiate_answer("not an sdp").expect_err("garbage offer must fail");
        assert!(matches!(err, SpikeError::InvalidOffer(_)));
    }
}
