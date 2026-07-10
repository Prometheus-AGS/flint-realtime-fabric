//! The sovereign str0m media engine — the `MediaTransport` implementation.
//!
//! Builds on the p19-c006 transport-loop proof (`transport_spike.rs`). Since p29-c001 (ADR-008)
//! the engine uses **one shared `UdpSocket`** owned by the transport and demultiplexed to every
//! session's `Rtc` by `Rtc::accepts()` in a single owning task (`demux.rs`) — replacing the
//! phase-20 per-session socket + driver, which collided on a fixed media port (phase-28 B2).
//! `create_session` negotiates a session's `Rtc` against the shared socket's bound address and
//! hands it to the demux loop; the per-session command/state/signal/forward channels are
//! unchanged, so the `MediaTransport` surface and the `RoomRouter` fan-out are untouched.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use dashmap::DashMap;
use frf_domain::{SessionId, SignalEnvelope, TenantId};
use frf_ports::{ConnectionState, MediaTransport, PortError, SignalStream};
use str0m::{Candidate, Rtc};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, watch};
use tokio_stream::StreamExt as _;

use crate::demux::{DemuxControl, DemuxSession, run_demux};
use crate::driver::{SessionCommand, SessionMeta};
use crate::error::StrOmError;

/// str0m's DTLS needs a process-wide crypto provider installed exactly once. Guard with a
/// `Once` so constructing multiple `StrOmTransport`s (or a test + the gateway) is safe.
static CRYPTO_INIT: std::sync::Once = std::sync::Once::new();

fn install_crypto_provider() {
    CRYPTO_INIT.call_once(|| {
        str0m::crypto::from_feature_flags().install_process_default();
    });
}

/// Handle the registry keeps for each live session.
struct SessionHandle {
    cmd_tx: mpsc::Sender<SessionCommand>,
    state_rx: watch::Receiver<ConnectionState>,
    /// Outbound trickle-ICE / state envelopes to relay over the signaling path.
    local_signals_tx: broadcast::Sender<SignalEnvelope>,
    /// The session's local host candidate envelope, captured at bind. Prepended to every
    /// `local_signals` subscription so a subscriber always sees it regardless of timing
    /// (a `broadcast` channel does not replay past sends to late subscribers).
    host_candidate: SignalEnvelope,
    /// Sender the room router pushes forwarded media into; the driver writes it to the `Rtc`.
    forward_tx: mpsc::Sender<crate::room::ForwardedFrame>,
}

/// The lazily-bound shared media socket + demux loop (ADR-008). Bound once, on the first
/// `create_session`, so the sync `with_config` constructor stays usable from non-async tests.
struct SharedDemux {
    /// Bound address of the one shared socket, used to advertise each session's host candidate.
    local_addr: SocketAddr,
    /// Control channel into the demux loop: register / drop a session's `Rtc`.
    control_tx: mpsc::Sender<DemuxControl>,
}

/// Sovereign SFU media engine: **one** shared `UdpSocket` demultiplexed to every session's `Rtc`
/// by `Rtc::accepts()` (ADR-008, p29-c001), implementing [`MediaTransport`]. Signaling stays in
/// `StrOmSignaler` (a separate port).
pub struct StrOmTransport {
    sessions: Arc<DashMap<SessionId, SessionHandle>>,
    /// Central room registry + RTP fan-out router shared with the demux loop (ADR-006).
    router: Arc<crate::room::RoomRouter>,
    /// Network bind/advertise configuration (p24-c001). Defaults to loopback-ephemeral.
    config: crate::config::MediaConfig,
    /// The shared socket + demux loop, bound lazily on first use.
    demux: tokio::sync::OnceCell<SharedDemux>,
}

impl Default for StrOmTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl StrOmTransport {
    /// Loopback-ephemeral transport (the historical default) — used by every in-process test.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(crate::config::MediaConfig::loopback())
    }

    /// Transport with an explicit [`MediaConfig`] (p24-c001). For `SFU_MODE=sovereign` the
    /// gateway supplies a `0.0.0.0` bind + advertised host IP + fixed UDP port so a browser can
    /// reach the media socket across process/container boundaries.
    #[must_use]
    pub fn with_config(config: crate::config::MediaConfig) -> Self {
        // Install the crypto provider so DTLS can key. Idempotent across constructions.
        install_crypto_provider();
        Self {
            sessions: Arc::new(DashMap::new()),
            router: Arc::new(crate::room::RoomRouter::new()),
            config,
            demux: tokio::sync::OnceCell::new(),
        }
    }

    /// Bind the one shared media socket and spawn the demux loop, exactly once. Returns the
    /// shared bound address + control channel used to register each session's `Rtc` (ADR-008).
    async fn ensure_demux(&self) -> Result<&SharedDemux, StrOmError> {
        self.demux
            .get_or_try_init(|| async {
                let socket = UdpSocket::bind((self.config.bind_addr, self.config.udp_port))
                    .await
                    .map_err(|e| StrOmError::Transport(format!("shared udp bind: {e}")))?;
                let local_addr = socket
                    .local_addr()
                    .map_err(|e| StrOmError::Transport(format!("local_addr: {e}")))?;
                let (control_tx, control_rx) = mpsc::channel(32);
                tokio::spawn(run_demux(
                    socket,
                    control_rx,
                    Arc::clone(&self.router),
                ));
                Ok(SharedDemux {
                    local_addr,
                    control_tx,
                })
            })
            .await
    }

    /// Move a session into `room` so its media fans out to that room's other members. The
    /// default room (set at `create_session`) is the session's own id; `join_room` regroups
    /// it. Used by the gateway (and the 1-to-1 forwarding test) to co-locate peers.
    pub fn join_room(&self, session_id: SessionId, tenant_id: TenantId, room: &str) {
        if let Some(entry) = self.sessions.get(&session_id) {
            self.router
                .register(session_id, tenant_id, room, entry.forward_tx.clone());
        }
    }

    /// Test-only access to the room router, to assert create/join/remove wiring.
    #[cfg(test)]
    pub(crate) fn router(&self) -> &crate::room::RoomRouter {
        &self.router
    }

    /// Await a session reaching [`ConnectionState::Connected`] (DTLS handshake complete),
    /// giving up after `timeout`. Returns the terminal state observed: `Connected` on
    /// success, or the last state (`Failed`/`Disconnected`/`Connecting`/`New`) on timeout.
    ///
    /// # Errors
    ///
    /// [`PortError::Transport`] if the session is unknown.
    pub async fn wait_for_connected(
        &self,
        session_id: SessionId,
        timeout: std::time::Duration,
    ) -> Result<ConnectionState, PortError> {
        let mut rx = {
            let entry = self
                .sessions
                .get(&session_id)
                .ok_or_else(|| PortError::Transport("unknown session".to_owned()))?;
            entry.state_rx.clone()
        };
        let wait = async {
            loop {
                let state = *rx.borrow_and_update();
                if matches!(
                    state,
                    ConnectionState::Connected
                        | ConnectionState::Failed
                        | ConnectionState::Disconnected
                ) {
                    return state;
                }
                if rx.changed().await.is_err() {
                    // The driver dropped its sender — the session ended.
                    return ConnectionState::Disconnected;
                }
            }
        };
        Ok(tokio::time::timeout(timeout, wait)
            .await
            .unwrap_or_else(|_| *rx.borrow()))
    }

    /// Negotiate the answer for `offer_sdp` against the **shared** socket's bound address,
    /// returning the started `Rtc`, the answer SDP, the local **host candidate**'s SDP string
    /// (relayed outbound as trickle ICE), and the advertised `SocketAddr`. No per-session bind
    /// (ADR-008) — the session's `Rtc` advertises the shared socket's address and is
    /// demultiplexed by `Rtc::accepts()`. The advertised addr is stored in `DemuxSession` so
    /// `route_datagram` can construct `Receive` with the correct destination (p36-c002f).
    fn negotiate(
        &self,
        local_addr: SocketAddr,
        offer_sdp: &str,
    ) -> Result<(Rtc, String, String, SocketAddr), StrOmError> {
        // The candidate advertised to the remote peer: resolve the configured advertise host to a
        // concrete IP (an IP as-is; a hostname like `host.docker.internal` via DNS) — the bind
        // address is often `0.0.0.0`, which str0m rejects as an ICE candidate. Fall back to the
        // shared bound address only when no advertise host is configured (loopback/dev). p28-c002.
        let advertised_addr = match self
            .config
            .resolve_advertised_ip()
            .map_err(|e| StrOmError::Transport(format!("advertise host: {e}")))?
        {
            Some(ip) => SocketAddr::new(ip, local_addr.port()),
            None => local_addr,
        };

        // p36-c002e: SFUs should run ICE-lite — the browser (ICE-full, controlling) does all
        // the checking and nominates; str0m responds to every STUN request with SUCCESS_RESPONSE.
        // p36-c002f: route_datagram must pass advertised_addr (not the socket bind address
        // 0.0.0.0) as the Receive destination. str0m's ICE agent validates incoming STUN
        // requests against local candidate addresses; the host candidate is 172.18.0.6:40000 so
        // a destination of 0.0.0.0:40000 never matches — all 241 STUN checks were silently
        // discarded (coturn peer rp=0). DemuxSession now carries advertised_addr for this.
        let mut rtc = Rtc::builder().set_ice_lite(true).build(Instant::now());
        let candidate = Candidate::host(advertised_addr, "udp")
            .map_err(|e| StrOmError::Transport(format!("host candidate: {e}")))?;
        let host_candidate = candidate.to_sdp_string();
        rtc.add_local_candidate(candidate);

        let offer = str0m::change::SdpOffer::from_sdp_string(offer_sdp)
            .map_err(|e| StrOmError::Negotiation(format!("invalid offer: {e}")))?;
        let answer = rtc
            .sdp_api()
            .accept_offer(offer)
            .map_err(|e| StrOmError::Negotiation(e.to_string()))?;

        Ok((rtc, answer.to_sdp_string(), host_candidate, advertised_addr))
    }
}

#[async_trait]
impl MediaTransport for StrOmTransport {
    #[tracing::instrument(name = "str0m::create_session", skip(self, offer_sdp))]
    async fn create_session(
        &self,
        session_id: SessionId,
        tenant_id: TenantId,
        offer_sdp: &str,
    ) -> Result<String, PortError> {
        // Bind the shared socket + demux loop on first use, then negotiate this session's `Rtc`
        // against the shared bound address (ADR-008 — no per-session bind).
        let local_addr = self.ensure_demux().await?.local_addr;
        let (rtc, answer, host_candidate, advertised_addr) =
            self.negotiate(local_addr, offer_sdp)?;
        // Lifecycle visibility (p27-c001): the run shows the offer was accepted and which host
        // candidate the SFU advertised — the first checkpoint when diagnosing an ICE stall.
        tracing::info!(
            %session_id, bind = %local_addr, advertised = %host_candidate,
            "sovereign: session negotiated (offer accepted, host candidate advertised)"
        );

        let (cmd_tx, cmd_rx) = mpsc::channel(16);
        let (state_tx, state_rx) = watch::channel(ConnectionState::New);
        let (local_signals_tx, _) = broadcast::channel(16);
        // Bounded forwarding channel: a full channel drops the frame (RTP is loss-tolerant).
        let (forward_tx, forward_rx) = mpsc::channel(64);

        // The room is unknown to the transport port (the gateway stamps routing); outbound
        // envelopes carry the candidate + session identity with an empty room.
        let meta = SessionMeta {
            session_id,
            tenant_id,
            room_id: String::new(),
        };
        // The session's local host candidate, kept so every `local_signals` subscriber sees
        // it (a broadcast channel does not replay to late subscribers).
        let host_candidate_env =
            crate::ice::candidate_envelope(tenant_id, session_id, &meta.room_id, &host_candidate);

        // Register in the router under a default room (its own id) so it has a forwarding
        // channel from the start; `join_room` regroups it with peers.
        self.router.register(
            session_id,
            tenant_id,
            &session_id.to_string(),
            forward_tx.clone(),
        );

        // Hand the negotiated `Rtc` + its per-session channels to the shared demux loop, which
        // owns every `Rtc` and routes inbound datagrams by `accepts()` (ADR-008). The transport
        // keeps only the sender ends (in `SessionHandle`) to drive the session.
        let demux_session = Box::new(DemuxSession {
            rtc,
            meta,
            advertised_addr,
            state_tx,
            local_signals_tx: local_signals_tx.clone(),
            cmd_rx,
            forward_rx,
        });
        self.ensure_demux()
            .await?
            .control_tx
            .send(DemuxControl::Add(demux_session))
            .await
            .map_err(|_| PortError::Transport("demux loop gone".to_owned()))?;
        self.sessions.insert(
            session_id,
            SessionHandle {
                cmd_tx,
                state_rx,
                local_signals_tx,
                host_candidate: host_candidate_env,
                forward_tx,
            },
        );

        Ok(answer)
    }

    #[tracing::instrument(name = "str0m::add_remote_candidate", skip(self, candidate))]
    async fn add_remote_candidate(
        &self,
        session_id: SessionId,
        candidate: SignalEnvelope,
    ) -> Result<(), PortError> {
        let entry = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| PortError::Transport("unknown session".to_owned()))?;
        // Extract the candidate string from the IceCandidate envelope (bare string or a
        // `{"candidate": …}` object — the browser shape).
        let cand = crate::ice::candidate_string_from_envelope(&candidate)
            .ok_or_else(|| PortError::Transport("envelope carries no ICE candidate".to_owned()))?;
        entry
            .cmd_tx
            .send(SessionCommand::AddRemoteCandidate(cand))
            .await
            .map_err(|_| PortError::Transport("session driver gone".to_owned()))?;
        Ok(())
    }

    async fn local_signals(&self, session_id: SessionId) -> Result<SignalStream, PortError> {
        let entry = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| PortError::Transport("unknown session".to_owned()))?;
        let rx = entry.local_signals_tx.subscribe();
        // Prepend the host candidate (captured at bind) so a subscriber always sees it,
        // then the live broadcast of subsequent local candidates + state changes.
        let host = tokio_stream::iter(std::iter::once(Ok(entry.host_candidate.clone())));
        let live =
            tokio_stream::wrappers::BroadcastStream::new(rx).filter_map(|item| item.ok().map(Ok));
        Ok(Box::pin(host.chain(live)))
    }

    #[tracing::instrument(name = "str0m::connection_state", skip(self))]
    async fn connection_state(&self, session_id: SessionId) -> Result<ConnectionState, PortError> {
        let entry = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| PortError::Transport("unknown session".to_owned()))?;
        Ok(*entry.state_rx.borrow())
    }

    #[tracing::instrument(name = "str0m::remove_session", skip(self))]
    async fn remove_session(
        &self,
        session_id: SessionId,
        _tenant_id: TenantId,
    ) -> Result<(), PortError> {
        self.router.deregister(session_id);
        if self.sessions.remove(&session_id).is_some() {
            // Tell the demux loop to drop the session's `Rtc` immediately (ADR-008). If the demux
            // loop was never started (no session ever created) there is nothing to remove.
            if let Some(demux) = self.demux.get() {
                let _ = demux
                    .control_tx
                    .send(DemuxControl::Remove(session_id))
                    .await;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "session_tests.rs"]
mod tests;
