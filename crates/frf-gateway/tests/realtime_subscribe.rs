#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Browser realtime contract on `/ws/v1/subscribe`: single-use tickets with a
//! `topics` claim (F2), resume from a client offset with resync (F1), and
//! presence heartbeats with expiry (F3). Runs against an in-memory broker.

#[path = "support/realtime.rs"]
mod realtime;

use std::time::Duration;

use frf_domain::{EventEnvelope, EventKind, Presence, PresenceStatus};
use frf_ports::LogBroker as _;
use futures_util::{SinkExt as _, StreamExt as _};
use realtime::{Fixture, channel_for, envelope, spawn, ticket_claims};
use serde_json::Value;
use tokio_tungstenite::tungstenite::{self, Message, client::IntoClientRequest as _};

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

const WAIT: Duration = Duration::from_secs(5);
const ROSTER: &str = "t/acme/u/user-1/roster";
const PRESENCE: &str = "t/acme/c/chan-1/presence";

fn url(fixture: &Fixture, channel: &str, query: &str) -> String {
    format!(
        "ws://{}/ws/v1/subscribe?channel={channel}{query}",
        fixture.address
    )
}

/// Register a fresh ticket for `topics` and return its token string.
fn ticket(fixture: &Fixture, subject: &str, topics: &[&str]) -> String {
    let token = format!("ticket-{}", uuid::Uuid::new_v4());
    fixture
        .tokens
        .insert(&token, ticket_claims(subject, 60, topics));
    token
}

async fn connect(url: &str) -> Result<Socket, u16> {
    match tokio_tungstenite::connect_async(url).await {
        Ok((socket, _)) => Ok(socket),
        Err(tungstenite::Error::Http(response)) => Err(response.status().as_u16()),
        Err(other) => panic!("unexpected connect error: {other}"),
    }
}

async fn next_json(socket: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(WAIT, socket.next())
            .await
            .expect("frame deadline")
            .expect("socket open")
            .expect("frame");
        if let Message::Text(text) = message {
            return serde_json::from_str(text.as_str()).expect("json frame");
        }
    }
}

async fn next_envelope(socket: &mut Socket) -> EventEnvelope {
    serde_json::from_value(next_json(socket).await).expect("EventEnvelope frame")
}

#[tokio::test]
async fn ticket_opens_one_subscription_and_is_refused_on_reuse() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);
    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let address = url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={token}"),
    );

    let mut socket = connect(&address).await.expect("first use opens");
    fixture.broker.publish(envelope(&channel, 1)).await.unwrap();
    let received = next_envelope(&mut socket).await;
    assert_eq!(received.channel.path, ROSTER);
    assert_eq!(received.payload["n"], 1);

    assert_eq!(connect(&address).await.err(), Some(401), "replayed ticket");
}

#[tokio::test]
async fn topic_outside_the_claim_is_forbidden() {
    let fixture = spawn().await;
    let other = channel_for("t/acme/u/user-2/roster");
    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let address = url(&fixture, &other.id.to_string(), &format!("&ticket={token}"));
    assert_eq!(connect(&address).await.err(), Some(403));
}

#[tokio::test]
async fn long_lived_or_unidentified_tickets_are_refused() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);

    fixture
        .tokens
        .insert("long", ticket_claims("user-1", 61, &[ROSTER]));
    let mut no_jti = ticket_claims("user-1", 60, &[ROSTER]);
    no_jti.token_id = None;
    fixture.tokens.insert("no-jti", no_jti);
    let mut no_iat = ticket_claims("user-1", 60, &[ROSTER]);
    no_iat.issued_at = None;
    fixture.tokens.insert("no-iat", no_iat);

    for token in ["long", "no-jti", "no-iat"] {
        let address = url(
            &fixture,
            &channel.id.to_string(),
            &format!("&ticket={token}"),
        );
        assert_eq!(connect(&address).await.err(), Some(401), "{token}");
    }
}

#[tokio::test]
async fn ticket_in_subprotocol_is_accepted_and_frf_v1_is_echoed() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);
    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let mut request = url(&fixture, &channel.id.to_string(), "")
        .into_client_request()
        .unwrap();
    request.headers_mut().insert(
        "sec-websocket-protocol",
        format!("frf.v1, ticket.{token}").parse().unwrap(),
    );
    let (_socket, response) = tokio_tungstenite::connect_async(request)
        .await
        .expect("subprotocol ticket opens");
    assert_eq!(
        response.headers().get("sec-websocket-protocol").unwrap(),
        "frf.v1"
    );
}

#[tokio::test]
async fn resume_from_offset_delivers_exactly_the_later_events() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);
    for n in 0..2 {
        fixture.broker.publish(envelope(&channel, n)).await.unwrap();
    }
    let last_applied = 1;
    for n in 2..5 {
        fixture.broker.publish(envelope(&channel, n)).await.unwrap();
    }

    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let from = last_applied + 1;
    let address = url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={token}&from={from}"),
    );
    let mut socket = connect(&address).await.expect("resume opens");
    let offsets: Vec<u64> = [
        next_envelope(&mut socket).await,
        next_envelope(&mut socket).await,
        next_envelope(&mut socket).await,
    ]
    .iter()
    .map(|e| e.offset.0)
    .collect();
    assert_eq!(offsets, vec![2, 3, 4]);
    let nothing_more = tokio::time::timeout(Duration::from_millis(300), socket.next()).await;
    assert!(
        nothing_more.is_err(),
        "no frame beyond the three later events"
    );
}

#[tokio::test]
async fn offset_below_retention_resyncs_then_streams_live_from_head() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);
    for n in 0..5 {
        fixture.broker.publish(envelope(&channel, n)).await.unwrap();
    }
    fixture.broker.expire_before(3);

    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let address = url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={token}&from=1"),
    );
    let mut socket = connect(&address).await.expect("stale offset still opens");
    let resync = next_json(&mut socket).await;
    assert_eq!(resync["type"], "resync");
    assert_eq!(resync["channel"], channel.id.to_string());
    assert_eq!(resync["head"], 4);

    fixture.broker.publish(envelope(&channel, 5)).await.unwrap();
    let live = next_envelope(&mut socket).await;
    assert_eq!(
        live.offset.0, 5,
        "first frame after resync is live, after head"
    );
}

#[tokio::test]
async fn non_numeric_from_is_a_bad_request() {
    let fixture = spawn().await;
    let channel = channel_for(ROSTER);
    let token = ticket(&fixture, "user-1", &[ROSTER]);
    let address = url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={token}&from=abc"),
    );
    assert_eq!(connect(&address).await.err(), Some(400));
}

#[tokio::test]
async fn heartbeat_shows_online_to_members_and_expiry_shows_offline() {
    let fixture = spawn().await;
    let channel = channel_for(PRESENCE);
    let watcher_ticket = ticket(&fixture, "user-2", &[PRESENCE]);
    let mut watcher = connect(&url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={watcher_ticket}"),
    ))
    .await
    .expect("watcher opens");

    let member_ticket = ticket(&fixture, "user-1", &[PRESENCE]);
    let mut member = connect(&url(
        &fixture,
        &channel.id.to_string(),
        &format!("&ticket={member_ticket}"),
    ))
    .await
    .expect("member opens");

    let heartbeat = r#"{"type":"presence.heartbeat","user_id":"spoofed"}"#;
    member.send(Message::text(heartbeat)).await.unwrap();
    let online = next_envelope(&mut watcher).await;
    assert_eq!(online.kind, EventKind::Presence);
    assert_eq!(online.channel.path, PRESENCE);
    let presence: Presence = serde_json::from_value(online.payload).unwrap();
    assert_eq!(
        presence.user_id, "user-1",
        "user id comes from the verified sub"
    );
    assert_eq!(presence.status, PresenceStatus::Online);
    let ttl = presence.expires_at - presence.last_seen;
    assert_eq!(ttl.num_seconds(), 45);

    // A repeat heartbeat refreshes without announcing again.
    member.send(Message::text(heartbeat)).await.unwrap();
    tokio::time::sleep(Duration::from_millis(200)).await;
    let presence_events = || {
        fixture
            .broker
            .published()
            .into_iter()
            .filter(|e| e.kind == EventKind::Presence)
            .count()
    };
    assert_eq!(presence_events(), 1);

    let swept = fixture
        .pipeline
        .presence()
        .sweep_at(presence.last_seen + chrono::TimeDelta::seconds(44))
        .await
        .unwrap();
    assert_eq!(swept, 0, "still online 44 s after the last heartbeat");
    let swept = fixture
        .pipeline
        .presence()
        .sweep_at(presence.last_seen + chrono::TimeDelta::seconds(46))
        .await
        .unwrap();
    assert_eq!(swept, 1);
    let offline = next_envelope(&mut watcher).await;
    let presence: Presence = serde_json::from_value(offline.payload).unwrap();
    assert_eq!(presence.user_id, "user-1");
    assert_eq!(presence.status, PresenceStatus::Offline);
}
