use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, SecondsFormat};
use frf_domain::{ChannelId, EntityChange, EntityId, EventEnvelope, EventKind, Offset};
use frf_ports::{EntityStore, LogBroker, ProjectionCursor};
use frf_postgres_cdc::model::{CanonicalValue, CdcMutation};
use futures_util::StreamExt as _;
use sha2::{Digest as _, Sha256};
use tokio::sync::watch;
use uuid::Uuid;

const CONSUMER_ID: &str = "entity-projection-v1";

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum EntityProjectorError {
    #[error("entity projection broker error: {0}")]
    Broker(#[from] frf_ports::PortError),
    #[error("entity projection payload is invalid: {0}")]
    Payload(String),
    #[error("entity projection stream ended unexpectedly")]
    StreamEnded,
}

/// Durable broker-to-entity projection worker.
pub struct EntityProjector<L, S: ?Sized> {
    broker: Arc<L>,
    store: Arc<S>,
}

impl<L, S> EntityProjector<L, S>
where
    L: LogBroker,
    S: EntityStore + ?Sized,
{
    #[must_use]
    pub fn new(broker: Arc<L>, store: Arc<S>) -> Self {
        Self { broker, store }
    }

    /// Replay from the inclusive durable checkpoint, then follow live events.
    ///
    /// # Errors
    ///
    /// Returns an error for broker failure, malformed CDC payload, source history
    /// mismatch, missing update base state, or unexpected stream termination.
    pub async fn run_until_shutdown(
        &self,
        mut shutdown: watch::Receiver<bool>,
        readiness: watch::Sender<bool>,
    ) -> Result<(), EntityProjectorError> {
        let _guard = ReadinessGuard(Some(readiness.clone()));
        let from = self
            .store
            .projection_checkpoint()
            .await?
            .map_or(Offset::BEGINNING, |cursor| Offset(cursor.broker_offset));
        let mut stream = self
            .broker
            .subscribe(ChannelId::WELL_KNOWN_ENTITIES, CONSUMER_ID.to_owned(), from)
            .await?;
        let mut catch_up_to = self
            .broker
            .head_offset(ChannelId::WELL_KNOWN_ENTITIES)
            .await?;
        if catch_up_to.is_none() {
            readiness.send_replace(true);
        }
        loop {
            tokio::select! {
                changed = shutdown.changed() => {
                    if changed.is_err() || *shutdown.borrow() {
                        return Ok(());
                    }
                }
                item = stream.next() => {
                    let envelope = item.ok_or(EntityProjectorError::StreamEnded)??;
                    let delivered = envelope.offset;
                    self.apply_envelope(envelope).await?;
                    if catch_up_to.is_some_and(|head| delivered >= head) {
                        readiness.send_replace(true);
                        catch_up_to = None;
                    }
                }
            }
        }
    }

    async fn apply_envelope(&self, envelope: EventEnvelope) -> Result<(), EntityProjectorError> {
        if envelope.kind != EventKind::EntityChange {
            return Err(EntityProjectorError::Payload(
                "well-known entity channel carried a non-entity event".to_owned(),
            ));
        }
        let mutation: CdcMutation = serde_json::from_value(envelope.payload)
            .map_err(|error| EntityProjectorError::Payload(error.to_string()))?;
        if mutation.tenant_id != envelope.channel.tenant_id {
            return Err(EntityProjectorError::Payload(
                "envelope tenant does not match committed mutation tenant".to_owned(),
            ));
        }
        let cursor = ProjectionCursor {
            source_epoch: mutation.source.epoch.clone(),
            commit_lsn: mutation.source.commit_lsn,
            transaction_index: mutation.source.transaction_index,
            broker_offset: envelope.offset.0,
        };
        let change = mutation_to_change(mutation, envelope.offset)?;
        self.store.apply_projection(change, cursor).await?;
        self.broker
            .ack(ChannelId::WELL_KNOWN_ENTITIES, CONSUMER_ID, envelope.offset)
            .await?;
        Ok(())
    }
}

fn mutation_to_change(
    mutation: CdcMutation,
    broker_offset: Offset,
) -> Result<EntityChange, EntityProjectorError> {
    let mut data = serde_json::Map::new();
    for field in mutation.record.unwrap_or_default() {
        data.insert(field.column, canonical_to_json(field.value)?);
    }
    Ok(EntityChange {
        entity_id: stable_entity_id(&mutation.key.canonical_id),
        tenant_id: mutation.tenant_id,
        entity_type: mutation.entity_type,
        op: mutation.op,
        data: serde_json::Value::Object(data),
        previous: None,
        session_id: None,
        timestamp: mutation.committed_at,
        version: broker_offset.0,
    })
}

fn stable_entity_id(canonical_key: &str) -> EntityId {
    let digest = Sha256::digest(canonical_key.as_bytes());
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    EntityId::from_uuid(Uuid::from_bytes(bytes))
}

fn canonical_to_json(value: CanonicalValue) -> Result<serde_json::Value, EntityProjectorError> {
    let value = match value {
        CanonicalValue::Null => serde_json::Value::Null,
        CanonicalValue::Bool(value) => value.into(),
        CanonicalValue::SignedInteger(value) => value.into(),
        CanonicalValue::UnsignedInteger(value) => value.into(),
        CanonicalValue::Float(value) => serde_json::Number::from_f64(value)
            .map(serde_json::Value::Number)
            .ok_or_else(|| {
                EntityProjectorError::Payload("non-finite float is not valid JSON".to_owned())
            })?,
        CanonicalValue::Text(value)
        | CanonicalValue::Bytes(value)
        | CanonicalValue::Decimal(value)
        | CanonicalValue::Uuid(value)
        | CanonicalValue::Date(value)
        | CanonicalValue::Time(value)
        | CanonicalValue::TimestampWithoutZone(value) => value.into(),
        CanonicalValue::Timestamp { seconds, nanos } => {
            let nanos = u32::try_from(nanos).map_err(|error| {
                EntityProjectorError::Payload(format!("timestamp nanos: {error}"))
            })?;
            DateTime::from_timestamp(seconds, nanos)
                .ok_or_else(|| {
                    EntityProjectorError::Payload("timestamp is out of range".to_owned())
                })?
                .to_rfc3339_opts(SecondsFormat::AutoSi, true)
                .into()
        }
        CanonicalValue::Json(value) => {
            let bytes = URL_SAFE_NO_PAD
                .decode(value)
                .map_err(|error| EntityProjectorError::Payload(error.to_string()))?;
            serde_json::from_slice(&bytes)
                .map_err(|error| EntityProjectorError::Payload(error.to_string()))?
        }
        _ => {
            return Err(EntityProjectorError::Payload(
                "unsupported future canonical value".to_owned(),
            ));
        }
    };
    Ok(value)
}

struct ReadinessGuard(Option<watch::Sender<bool>>);

impl Drop for ReadinessGuard {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            sender.send_replace(false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_entity_id_is_deterministic_and_uuid_shaped() {
        let first = stable_entity_id("frfkey:v1:abc");
        let retry = stable_entity_id("frfkey:v1:abc");
        let other = stable_entity_id("frfkey:v1:def");
        assert_eq!(first, retry);
        assert_ne!(first, other);
    }

    #[test]
    fn canonical_json_is_decoded_from_jcs_bytes() {
        let encoded = URL_SAFE_NO_PAD.encode(br#"{"a":1}"#);
        assert_eq!(
            canonical_to_json(CanonicalValue::Json(encoded)).expect("decode canonical JSON"),
            serde_json::json!({"a": 1})
        );
    }

    #[test]
    fn non_finite_float_is_rejected() {
        assert!(matches!(
            canonical_to_json(CanonicalValue::Float(f64::NAN)),
            Err(EntityProjectorError::Payload(message))
                if message.contains("non-finite float")
        ));
    }
}
