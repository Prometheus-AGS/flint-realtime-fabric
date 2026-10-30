//! Single-use browser ticket rules for the subscribe pipeline.
//!
//! A ticket is a short-lived JWT that a browser presents on the WebSocket
//! upgrade instead of an `Authorization` header. It is verified exactly like a
//! bearer token; these rules are applied on top:
//!
//! - it carries a `jti`, and a `jti` is accepted once until the ticket expires;
//! - it carries `iat`, and `exp - iat` is at most [`MAX_TICKET_LIFETIME_SECS`];
//! - a `topics` claim, when present, names the only topic paths it may read.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use frf_domain::ChannelId;
use frf_ports::{PortError, VerifiedClaims};
use uuid::Uuid;

use crate::error::AppError;

/// Longest accepted ticket lifetime, `exp - iat`, in seconds.
pub const MAX_TICKET_LIFETIME_SECS: u64 = 60;

/// How a subscribe credential reached the gateway.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialKind {
    /// `Authorization: Bearer <jwt>`.
    Bearer,
    /// A single-use browser ticket (query parameter or WebSocket subprotocol).
    Ticket,
}

/// Remembers ticket ids until their expiry so each is accepted once.
///
/// Process-local: a deployment with several gateway replicas accepts a ticket
/// once per replica.
#[derive(Debug, Default)]
pub(crate) struct TicketReplayGuard {
    seen: Mutex<HashMap<String, u64>>,
}

impl TicketReplayGuard {
    /// Record `jti` until `expires_at`. Returns `false` when it was already used.
    pub(crate) fn claim(&self, jti: &str, expires_at: u64, now: u64) -> bool {
        let mut seen = self.seen.lock().unwrap_or_else(PoisonError::into_inner);
        seen.retain(|_, expiry| *expiry > now);
        if seen.contains_key(jti) {
            return false;
        }
        seen.insert(jti.to_owned(), expires_at);
        true
    }
}

/// Apply the single-use and lifetime rules to verified ticket claims.
pub(crate) fn admit_ticket(
    claims: &VerifiedClaims,
    replay: &TicketReplayGuard,
    now: u64,
) -> Result<(), AppError> {
    let jti = claims
        .token_id
        .as_deref()
        .filter(|jti| !jti.is_empty())
        .ok_or_else(|| unauthorized("ticket has no jti"))?;
    let issued_at = claims
        .issued_at
        .ok_or_else(|| unauthorized("ticket has no iat"))?;
    if claims.expires_at.saturating_sub(issued_at) > MAX_TICKET_LIFETIME_SECS {
        return Err(unauthorized("ticket lifetime exceeds 60 seconds"));
    }
    if !replay.claim(jti, claims.expires_at, now) {
        return Err(unauthorized("ticket already used"));
    }
    Ok(())
}

/// Find the listed topic whose `UUIDv5` under `namespace` is `channel_id`.
pub(crate) fn match_topic(
    topics: &[String],
    namespace: Option<Uuid>,
    channel_id: ChannelId,
) -> Result<String, AppError> {
    let namespace = namespace.ok_or_else(|| {
        AppError::Forbidden(
            "token carries a topics claim but no topic namespace is configured".to_owned(),
        )
    })?;
    topics
        .iter()
        .find(|topic| topic_channel_id(namespace, topic) == channel_id)
        .cloned()
        .ok_or_else(|| {
            AppError::Forbidden(format!("channel {channel_id} is not in the topics claim"))
        })
}

/// The channel id of a topic path: `UUIDv5` of the path (UTF-8) under `namespace`.
#[must_use]
pub fn topic_channel_id(namespace: Uuid, topic: &str) -> ChannelId {
    ChannelId::from_uuid(Uuid::new_v5(&namespace, topic.as_bytes()))
}

fn unauthorized(reason: &str) -> AppError {
    AppError::Identity(PortError::PermissionDenied(reason.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_guard_accepts_once_and_forgets_after_expiry() {
        let guard = TicketReplayGuard::default();
        assert!(guard.claim("a", 100, 50));
        assert!(!guard.claim("a", 100, 60));
        assert!(guard.claim("a", 200, 101));
    }

    #[test]
    fn topic_channel_id_is_uuid_v5_of_the_path() {
        // RFC 9562 Appendix A.4 test vector: v5 of "www.example.com" under the DNS namespace.
        let id = topic_channel_id(Uuid::NAMESPACE_DNS, "www.example.com");
        assert_eq!(id.to_string(), "2ed6657d-e927-568b-95e1-2665a8aea6a2");
    }
}
