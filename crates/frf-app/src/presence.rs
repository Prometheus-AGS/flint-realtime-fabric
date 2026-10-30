//! Presence heartbeats and expiry for subscribed channels.
//!
//! A subscriber with an open subscription heartbeats the channel it holds.
//! One record is kept per `(channel, user)`, so several tabs or a reconnect with
//! a fresh ticket refresh the same record. The first heartbeat, and any status
//! change, publishes an [`EventKind::Presence`] envelope on the channel; a
//! record with no heartbeat for [`PRESENCE_TTL`] is removed by
//! [`PresenceTracker::sweep_at`] and published once more as `offline`.
//!
//! [`PresenceTracker::snapshot_at`] reads the live records of a channel, so a
//! subscriber that opens after the last change still learns who is online.
//!
//! State is process-local: with several gateway replicas, each publishes the
//! presence of the subscribers connected to it, and a snapshot covers only the
//! records held by the replica serving the subscription.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use chrono::{DateTime, Utc};
use frf_domain::{Channel, ChannelId, EventEnvelope, EventKind, Offset, Presence, PresenceStatus};
use frf_ports::{LogBroker, PortError};
use tracing::instrument;

use crate::error::AppError;
use crate::subscribe::SubscriptionGrant;

/// How long a record stays online after its last heartbeat.
pub const PRESENCE_TTL: Duration = Duration::from_secs(45);
/// How often the gateway runs [`PresenceTracker::sweep_at`].
pub const PRESENCE_SWEEP_INTERVAL: Duration = Duration::from_secs(1);

type PresenceKey = (ChannelId, String);

struct Record {
    presence: Presence,
    path: String,
}

pub struct PresenceTracker<L> {
    broker: Arc<L>,
    records: Mutex<HashMap<PresenceKey, Record>>,
}

impl<L> PresenceTracker<L>
where
    L: LogBroker,
{
    #[must_use]
    pub fn new(broker: Arc<L>) -> Self {
        Self {
            broker,
            records: Mutex::new(HashMap::new()),
        }
    }

    /// Record a heartbeat at the current time. See [`Self::heartbeat_at`].
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Broker`] when the presence envelope cannot be published.
    pub async fn heartbeat(
        &self,
        grant: &SubscriptionGrant,
        status: PresenceStatus,
    ) -> Result<bool, AppError> {
        self.heartbeat_at(grant, status, Utc::now()).await
    }

    /// Refresh the grant holder's record on its channel, `expires_at = now + TTL`.
    ///
    /// Publishes, and returns `true`, when the record is new or its status
    /// changed. The `user_id` is the verified `sub` of the grant.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Broker`] when the presence envelope cannot be
    /// published; the record is then dropped so the next heartbeat retries.
    #[instrument(name = "app::presence::heartbeat", skip_all, fields(channel_id = %grant.channel_id))]
    pub async fn heartbeat_at(
        &self,
        grant: &SubscriptionGrant,
        status: PresenceStatus,
        now: DateTime<Utc>,
    ) -> Result<bool, AppError> {
        let key = (grant.channel_id, grant.subject.clone());
        let expires_at = now + ttl();
        let (announce, path) = {
            let mut records = self.lock();
            let changed = records
                .get(&key)
                .is_none_or(|record| record.presence.status != status);
            let path = grant.path();
            let record = records.entry(key.clone()).or_insert_with(|| Record {
                presence: Presence {
                    session_id: grant.session_id,
                    tenant_id: grant.tenant_id,
                    channel_id: grant.channel_id,
                    user_id: grant.subject.clone(),
                    display_name: None,
                    status: status.clone(),
                    meta: serde_json::Value::Object(serde_json::Map::new()),
                    last_seen: now,
                    expires_at,
                },
                path: path.clone(),
            });
            record.presence.session_id = grant.session_id;
            record.presence.status = status;
            record.presence.last_seen = now;
            record.presence.expires_at = expires_at;
            (changed.then(|| record.presence.clone()), path)
        };
        let Some(presence) = announce else {
            return Ok(false);
        };
        if let Err(error) = self.publish(presence, path).await {
            self.lock().remove(&key);
            return Err(error);
        }
        Ok(true)
    }

    /// Remove every record whose `expires_at` is not after `now` and publish it
    /// as `offline`. Returns how many were published.
    ///
    /// # Errors
    ///
    /// Returns the first publish failure; records that failed to publish are
    /// kept so the next sweep retries them.
    #[instrument(name = "app::presence::sweep", skip_all)]
    pub async fn sweep_at(&self, now: DateTime<Utc>) -> Result<usize, AppError> {
        let expired: Vec<(PresenceKey, Record)> = {
            let mut records = self.lock();
            let keys: Vec<PresenceKey> = records
                .iter()
                .filter(|(_, record)| record.presence.expires_at <= now)
                .map(|(key, _)| key.clone())
                .collect();
            keys.into_iter()
                .filter_map(|key| records.remove(&key).map(|record| (key, record)))
                .collect()
        };
        let mut published = 0;
        let mut first_error = None;
        for (key, record) in expired {
            let mut presence = record.presence.clone();
            presence.status = PresenceStatus::Offline;
            match self.publish(presence, record.path.clone()).await {
                Ok(()) => published += 1,
                Err(error) => {
                    self.lock().entry(key).or_insert(record);
                    first_error.get_or_insert(error);
                }
            }
        }
        first_error.map_or(Ok(published), Err)
    }

    /// The records on `channel_id` whose `expires_at` is after `now`, as
    /// [`EventKind::Presence`] envelopes ordered by `user_id`.
    ///
    /// The envelopes are not published and carry no broker position: their
    /// `offset` is [`Offset::BEGINNING`] as a placeholder only.
    ///
    /// # Errors
    ///
    /// Returns [`AppError::Broker`] when a record cannot be serialized.
    pub fn snapshot_at(
        &self,
        channel_id: ChannelId,
        now: DateTime<Utc>,
    ) -> Result<Vec<EventEnvelope>, AppError> {
        let mut live: Vec<(Presence, String)> = self
            .lock()
            .iter()
            .filter(|((channel, _), record)| {
                *channel == channel_id && record.presence.expires_at > now
            })
            .map(|(_, record)| (record.presence.clone(), record.path.clone()))
            .collect();
        live.sort_by(|(a, _), (b, _)| a.user_id.cmp(&b.user_id));
        live.into_iter()
            .map(|(presence, path)| presence_envelope(&presence, path))
            .collect()
    }

    async fn publish(&self, presence: Presence, path: String) -> Result<(), AppError> {
        let envelope = presence_envelope(&presence, path)?;
        self.broker.publish(envelope).await?;
        Ok(())
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<PresenceKey, Record>> {
        self.records.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn presence_envelope(presence: &Presence, path: String) -> Result<EventEnvelope, AppError> {
    let payload = serde_json::to_value(presence)
        .map_err(|error| AppError::Broker(PortError::Serialization(error.to_string())))?;
    Ok(EventEnvelope::new(
        Channel {
            id: presence.channel_id,
            tenant_id: presence.tenant_id,
            path,
        },
        Offset::BEGINNING,
        EventKind::Presence,
        payload,
    ))
}

fn ttl() -> chrono::TimeDelta {
    chrono::TimeDelta::from_std(PRESENCE_TTL).unwrap_or(chrono::TimeDelta::seconds(45))
}
