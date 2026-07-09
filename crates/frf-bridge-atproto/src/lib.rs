#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod convert;
pub mod error;
pub mod jetstream;
pub mod pds;

use async_trait::async_trait;
use frf_domain::ids::{ChannelId, TenantId};
use frf_ports::{
    error::PortError,
    federation::{FederationBridge, FederationProtocol, FederationStream},
};
use tracing::instrument;

use crate::pds::PdsClient;
pub use crate::pds::PdsConfig;

/// Default lexicon collection for outbound records when none is configured.
const DEFAULT_WRITE_COLLECTION: &str = "app.bsky.feed.post";

/// `FederationBridge` adapter for the `ATProto` / Bluesky protocol.
///
/// Inbound: connects to the Bluesky Jetstream WebSocket API and projects commit events
/// onto the FRF spine as `FederatedEvent`s. Outbound: when a PDS writer is configured
/// ([`AtProtoBridge::with_writer`]), `send` authenticates a session and writes a record
/// via `com.atproto.repo.createRecord`. Without a writer, `send` returns a clear error
/// (the bridge is inbound-only).
pub struct AtProtoBridge {
    jetstream_url: String,
    collections: Vec<String>,
    tenant_id: TenantId,
    channel_id: ChannelId,
    /// Outbound PDS writer + the collection to write into. `None` = inbound-only.
    writer: Option<(PdsClient, String)>,
}

impl AtProtoBridge {
    /// Create a new inbound-only `AtProtoBridge` that consumes from `jetstream_url`.
    ///
    /// `collections` filters events by `ATProto` lexicon type, e.g.
    /// `vec!["app.bsky.feed.post".to_owned()]`.
    #[must_use]
    pub fn new(
        jetstream_url: impl Into<String>,
        collections: Vec<String>,
        tenant_id: TenantId,
        channel_id: ChannelId,
    ) -> Self {
        Self {
            jetstream_url: jetstream_url.into(),
            collections,
            tenant_id,
            channel_id,
            writer: None,
        }
    }

    /// Enable the outbound write path with a PDS writer.
    ///
    /// `write_collection` is the lexicon type outbound records are written as; when `None`
    /// the default (`app.bsky.feed.post`) is used.
    #[must_use]
    pub fn with_writer(mut self, pds: PdsConfig, write_collection: Option<String>) -> Self {
        let collection = write_collection.unwrap_or_else(|| DEFAULT_WRITE_COLLECTION.to_owned());
        self.writer = Some((PdsClient::new(pds), collection));
        self
    }
}

#[async_trait]
impl FederationBridge for AtProtoBridge {
    #[instrument(skip(self, envelope, _destination), fields(protocol = ?protocol))]
    async fn send(
        &self,
        protocol: FederationProtocol,
        _destination: &str,
        envelope: frf_domain::EventEnvelope,
    ) -> Result<(), PortError> {
        if protocol != FederationProtocol::AtProto {
            return Err(PortError::Transport(format!(
                "AtProtoBridge cannot send {protocol:?} events"
            )));
        }
        let Some((pds, collection)) = self.writer.as_ref() else {
            return Err(PortError::Transport(
                "AtProtoBridge: outbound write not configured (inbound-only). \
                 Set the PDS identity to enable writes."
                    .to_owned(),
            ));
        };

        // The envelope payload IS the record body. Ensure it carries the lexicon `$type`
        // and a `createdAt` so it is a valid ATProto record, without clobbering caller
        // fields.
        let record = build_record(collection, envelope.payload, envelope.timestamp);
        pds.create_record(collection, record)
            .await
            .map_err(PortError::from)
    }

    #[instrument(skip(self), fields(protocol = ?protocol))]
    async fn subscribe(&self, protocol: FederationProtocol) -> Result<FederationStream, PortError> {
        if protocol != FederationProtocol::AtProto {
            return Err(PortError::Transport(format!(
                "AtProtoBridge cannot subscribe to {protocol:?} events"
            )));
        }

        let stream = jetstream::jetstream_stream(
            self.jetstream_url.clone(),
            self.collections.clone(),
            self.tenant_id,
            self.channel_id,
        );

        Ok(Box::pin(stream))
    }
}

/// Build an `ATProto` record from an event payload: ensure the lexicon `$type` and a
/// `createdAt` timestamp are present (an object payload is enriched in place; a non-object
/// payload is wrapped so the record is still valid). Caller-supplied `$type`/`createdAt`
/// are preserved.
fn build_record(
    collection: &str,
    payload: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
) -> serde_json::Value {
    let created_at = timestamp.to_rfc3339();
    match payload {
        serde_json::Value::Object(mut map) => {
            map.entry("$type")
                .or_insert_with(|| serde_json::Value::String(collection.to_owned()));
            map.entry("createdAt")
                .or_insert_with(|| serde_json::Value::String(created_at));
            serde_json::Value::Object(map)
        }
        other => serde_json::json!({
            "$type": collection,
            "createdAt": created_at,
            "value": other,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bridge() -> AtProtoBridge {
        AtProtoBridge::new(
            "wss://jetstream2.us-east.bsky.network/subscribe",
            vec!["app.bsky.feed.post".to_owned()],
            TenantId::new(),
            ChannelId::new(),
        )
    }

    #[tokio::test]
    async fn subscribe_returns_stream_for_atproto() {
        let bridge = make_bridge();
        // subscribe() should succeed — the stream itself may not yield events
        // in a unit test environment without a live Jetstream connection.
        let result = bridge.subscribe(FederationProtocol::AtProto).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn subscribe_rejects_wrong_protocol() {
        let bridge = make_bridge();
        let result = bridge.subscribe(FederationProtocol::Matrix).await;
        assert!(result.is_err());
    }

    use frf_domain::envelope::{Channel, EventEnvelope, EventKind, Offset};

    fn test_envelope(payload: serde_json::Value) -> EventEnvelope {
        EventEnvelope::new(
            Channel {
                id: ChannelId::new(),
                tenant_id: TenantId::new(),
                path: "test".to_owned(),
            },
            Offset::BEGINNING,
            EventKind::Custom("test".to_owned()),
            payload,
        )
    }

    #[tokio::test]
    async fn send_without_writer_reports_not_configured() {
        // An inbound-only bridge (no PDS writer) rejects sends with a clear message.
        let bridge = make_bridge();
        let err = bridge
            .send(
                FederationProtocol::AtProto,
                "dest",
                test_envelope(serde_json::json!({})),
            )
            .await
            .expect_err("inbound-only send should fail");
        assert!(err.to_string().contains("not configured"));
    }

    #[test]
    fn build_record_enriches_object_payload() {
        let ts = chrono::Utc::now();
        let record = build_record(
            "app.bsky.feed.post",
            serde_json::json!({ "text": "hello" }),
            ts,
        );
        assert_eq!(record["$type"], "app.bsky.feed.post");
        assert_eq!(record["text"], "hello");
        assert!(record["createdAt"].is_string());
    }

    #[test]
    fn build_record_preserves_caller_type_and_created_at() {
        let record = build_record(
            "app.bsky.feed.post",
            serde_json::json!({ "$type": "custom.type", "createdAt": "2020-01-01T00:00:00Z" }),
            chrono::Utc::now(),
        );
        assert_eq!(record["$type"], "custom.type");
        assert_eq!(record["createdAt"], "2020-01-01T00:00:00Z");
    }

    #[test]
    fn build_record_wraps_non_object_payload() {
        let record = build_record(
            "app.bsky.feed.post",
            serde_json::json!("plain"),
            chrono::Utc::now(),
        );
        assert_eq!(record["$type"], "app.bsky.feed.post");
        assert_eq!(record["value"], "plain");
    }

    #[tokio::test]
    async fn send_writes_record_to_pds() {
        use crate::pds::PdsConfig;
        use httpmock::MockServer;

        let server = MockServer::start_async().await;

        // Mock createSession → accessJwt + did.
        let session_mock = server.mock(|when, then| {
            when.method("POST")
                .path("/xrpc/com.atproto.server.createSession");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(serde_json::json!({ "accessJwt": "tok-123", "did": "did:plc:abc" }));
        });

        // Mock createRecord → 200, asserting it carried the DID as repo and our record.
        let record_mock = server.mock(|when, then| {
            when.method("POST")
                .path("/xrpc/com.atproto.repo.createRecord")
                .header("authorization", "Bearer tok-123")
                .json_body_partial(
                    r#"{ "repo": "did:plc:abc", "collection": "app.bsky.feed.post" }"#,
                );
            then.status(200)
                .json_body(serde_json::json!({ "uri": "at://did:plc:abc/x/1" }));
        });

        let bridge = make_bridge().with_writer(
            PdsConfig {
                service_url: server.base_url(),
                identifier: "alice.test".to_owned(),
                app_password: "app-pw".to_owned(),
            },
            None,
        );

        bridge
            .send(
                FederationProtocol::AtProto,
                "dest",
                test_envelope(serde_json::json!({ "text": "federated hello" })),
            )
            .await
            .expect("write should succeed");

        session_mock.assert();
        record_mock.assert();
    }
}
