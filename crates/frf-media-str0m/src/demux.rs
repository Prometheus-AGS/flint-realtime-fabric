//! The shared-socket demultiplexing loop for the sovereign SFU (ADR-008, p29-c001).
//!
//! Replaces the phase-20 per-session driver task (one `UdpSocket` + one `Rtc` per session,
//! which collided on a fixed media port — phase-28 blocker B2). One task owns **one** shared
//! `UdpSocket` and **all** live `Rtc`s; each inbound datagram is routed to its owning `Rtc`
//! via `Rtc::accepts()`, exactly as str0m's own `examples/chat.rs` demultiplexes clients.
//!
//! str0m is sans-I/O and single-threaded per `Rtc`, so a single owning task needs no locks
//! around the `Rtc` set (ADR-008, "single owning task" — the operator's p29 decision).

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use frf_domain::SessionId;
use frf_domain::SignalEnvelope;
use frf_ports::ConnectionState;
use str0m::net::{Protocol, Receive};
use str0m::{Candidate, Input, Output, Rtc};
use tokio::net::UdpSocket;
use tokio::sync::{broadcast, mpsc, watch};

use crate::driver::{RECV_BUF, SessionCommand, SessionMeta, apply_forwarded, handle_event};
use crate::room::{ForwardedFrame, RoomRouter};

/// One session's engine state, owned by the demux loop for the session's lifetime.
pub(crate) struct DemuxSession {
    pub(crate) rtc: Rtc,
    pub(crate) meta: SessionMeta,
    pub(crate) state_tx: watch::Sender<ConnectionState>,
    pub(crate) local_signals_tx: broadcast::Sender<SignalEnvelope>,
    pub(crate) cmd_rx: mpsc::Receiver<SessionCommand>,
    pub(crate) forward_rx: mpsc::Receiver<ForwardedFrame>,
}

/// Control messages the transport sends to the demux loop to add or drop a session.
pub(crate) enum DemuxControl {
    /// Register a freshly negotiated session's engine into the shared loop.
    Add(Box<DemuxSession>),
    /// Drop a session (best-effort; dropping its `cmd_rx` also ends servicing).
    Remove(SessionId),
}

/// Drain a single `Rtc`'s pending outputs, sending transmits over the shared socket and
/// publishing events. Returns the `Rtc`'s next timeout deadline, or `None` if the `Rtc`
/// has stopped (no further timeout — the session should be reaped).
async fn drive_outputs(
    session_id: SessionId,
    session: &mut DemuxSession,
    socket: &UdpSocket,
    router: &RoomRouter,
) -> Option<Instant> {
    loop {
        match session.rtc.poll_output() {
            Ok(Output::Timeout(t)) => return Some(t),
            Ok(Output::Transmit(t)) => {
                if let Err(e) = socket.send_to(&t.contents, t.destination).await {
                    tracing::warn!(session = %session_id, error = %e, "shared-socket transmit failed");
                }
            }
            Ok(Output::Event(event)) => {
                handle_event(
                    &event,
                    &session.state_tx,
                    &session.local_signals_tx,
                    router,
                    &session.meta,
                );
            }
            Err(e) => {
                tracing::warn!(session = %session_id, error = %e, "poll_output error — reaping session");
                let _ = session.state_tx.send(ConnectionState::Failed);
                return None;
            }
        }
    }
}

/// Feed a str0m command (remote candidate) into a session's `Rtc`.
fn apply_command(session_id: SessionId, session: &mut DemuxSession, cmd: SessionCommand) {
    match cmd {
        SessionCommand::AddRemoteCandidate(c) => match Candidate::from_sdp_string(&c) {
            Ok(cand) => session.rtc.add_remote_candidate(cand),
            // A browser host candidate may be an mDNS `.local` name str0m can't parse
            // (phase-28 B1). Skip that one candidate — never drop the whole session.
            Err(e) => {
                tracing::warn!(session = %session_id, error = %e, "bad remote candidate — skipping");
            }
        },
    }
}

/// The shared-socket demux loop. Owns `socket` and every live session's `Rtc`; routes inbound
/// datagrams by `Rtc::accepts()`, services each session's commands + forwarded media, and fires
/// each session's timers. Ends when `control_rx` closes (the transport was dropped).
pub(crate) async fn run_demux(
    socket: UdpSocket,
    local_addr: SocketAddr,
    mut control_rx: mpsc::Receiver<DemuxControl>,
    router: Arc<RoomRouter>,
) {
    let mut sessions: HashMap<SessionId, DemuxSession> = HashMap::new();
    let mut buf = vec![0_u8; RECV_BUF];

    loop {
        // 1. Drain every session's outputs; compute the earliest timer across all sessions and
        //    reap any that stopped.
        let mut earliest: Option<Instant> = None;
        let mut dead: Vec<SessionId> = Vec::new();
        // Collect ids first to avoid holding an iterator borrow across the await.
        let ids: Vec<SessionId> = sessions.keys().copied().collect();
        for id in ids {
            if let Some(session) = sessions.get_mut(&id) {
                match drive_outputs(id, session, &socket, &router).await {
                    Some(t) => earliest = Some(earliest.map_or(t, |e| e.min(t))),
                    None => dead.push(id),
                }
            }
        }
        for id in dead {
            reap(&mut sessions, &router, id);
        }

        // 2. Wait for: the earliest timer, an inbound datagram, or a control message. Per-session
        //    commands/forwarded media are polled by draining ready channels after each wakeup so a
        //    single loop fairly services every session.
        let sleep_until =
            earliest.unwrap_or_else(|| Instant::now() + std::time::Duration::from_millis(50));
        let now = Instant::now();
        let sleep = tokio::time::sleep(sleep_until.saturating_duration_since(now));

        tokio::select! {
            () = sleep => { /* timers fired below */ }
            recv = socket.recv_from(&mut buf) => {
                match recv {
                    Ok((n, source)) => route_datagram(&mut sessions, source, local_addr, &buf[..n]),
                    Err(e) => {
                        tracing::warn!(error = %e, "shared-socket recv_from error — ending demux");
                        return;
                    }
                }
            }
            ctrl = control_rx.recv() => match ctrl {
                Some(DemuxControl::Add(s)) => {
                    let id = s.meta.session_id;
                    sessions.insert(id, *s);
                }
                Some(DemuxControl::Remove(id)) => reap(&mut sessions, &router, id),
                None => return, // transport dropped
            }
        }

        // 3. Fire all sessions' timers and drain ready per-session channels (commands + forwarded
        //    media). Draining after every wakeup keeps one loop responsive across all sessions.
        service_sessions(&mut sessions, &router);
    }
}

/// Route one inbound datagram to the session whose `Rtc` accepts it (`Rtc::accepts`, ADR-008).
fn route_datagram(
    sessions: &mut HashMap<SessionId, DemuxSession>,
    source: SocketAddr,
    local_addr: SocketAddr,
    data: &[u8],
) {
    let Ok(receive) = Receive::new(Protocol::Udp, source, local_addr, data) else {
        tracing::warn!("malformed inbound datagram — skipping");
        return;
    };
    let input = Input::Receive(Instant::now(), receive);
    if let Some(session) = sessions.values_mut().find(|s| s.rtc.accepts(&input)) {
        if let Err(e) = session.rtc.handle_input(input) {
            tracing::warn!(error = %e, "handle_input error on routed datagram");
        }
    } else {
        tracing::debug!(%source, "no session accepts inbound datagram — dropping");
    }
}

/// Fire each session's timeout and drain its ready commands + forwarded media. A session whose
/// command channel signals shutdown (or closes) is reaped.
fn service_sessions(sessions: &mut HashMap<SessionId, DemuxSession>, router: &RoomRouter) {
    let ids: Vec<SessionId> = sessions.keys().copied().collect();
    for id in ids {
        let mut drop_it = false;
        if let Some(session) = sessions.get_mut(&id) {
            // Timer.
            if let Err(e) = session.rtc.handle_input(Input::Timeout(Instant::now())) {
                tracing::warn!(session = %id, error = %e, "timeout handle_input error — reaping");
                let _ = session.state_tx.send(ConnectionState::Failed);
                drop_it = true;
            }
            // Ready commands (non-blocking).
            while let Ok(cmd) = session.cmd_rx.try_recv() {
                apply_command(id, session, cmd);
            }
            // Ready forwarded media (non-blocking).
            while let Ok(frame) = session.forward_rx.try_recv() {
                apply_forwarded(&mut session.rtc, &frame);
            }
        }
        if drop_it {
            reap(sessions, router, id);
        }
    }
}

/// Remove a session from the loop and deregister it from the router.
fn reap(sessions: &mut HashMap<SessionId, DemuxSession>, router: &RoomRouter, id: SessionId) {
    if sessions.remove(&id).is_some() {
        router.deregister(id);
    }
}
