use std::collections::BTreeSet;

use async_trait::async_trait;
use frf_domain::TenantId;
use serde::{Deserialize, Serialize};

use crate::error::P2pError;

/// A peer's cryptographic identity, as presented during the handshake.
///
/// The `endpoint_id` is iroh's public key rendered as a string. It is *not* a
/// claim — iroh proves possession of the corresponding secret key as part of
/// establishing the QUIC session, so it is the one field an attacker cannot
/// forge. Everything else in [`PeerIdentity`] is asserted by the peer and is
/// worthless until [`TokenVerifier::verify`] validates it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerIdentity {
    /// The peer's iroh endpoint id (public key), proven by the QUIC handshake.
    pub endpoint_id: String,
    /// Tenant this peer belongs to, taken from verified token claims.
    pub tenant_id: TenantId,
    /// Subject (owner) claim from the verified token.
    pub subject: String,
}

/// Verifies the bearer token a peer presents during the handshake.
///
/// This is the seam where flint-gate's JWKS verification plugs in. The trait is
/// intentionally minimal so the fabric does not take a dependency on any
/// particular identity provider — callers supply an implementation.
///
/// # Errors
///
/// Implementations return [`P2pError::Unauthenticated`] when the token is
/// absent, malformed, expired, signed by an unknown key, or missing the claims
/// required to build a [`PeerIdentity`].
#[async_trait]
pub trait TokenVerifier: Send + Sync + std::fmt::Debug {
    /// Verify `token` and return the identity it attests to.
    ///
    /// `endpoint_id` is supplied so an implementation may additionally bind the
    /// token to the connecting key (preventing a valid token being replayed
    /// from a different device).
    async fn verify(&self, token: &str, endpoint_id: &str) -> Result<PeerIdentity, P2pError>;
}

/// A verifier that rejects every token.
///
/// This is the **default** when no verifier is configured, and it is what makes
/// the transport fail closed. Peer support ships enabled, but a node with no
/// configured verifier can accept no sessions: it is
/// enabled-but-unable-to-establish rather than enabled-and-trusting.
///
/// Using this type is not an error condition — it is the correct posture for a
/// node whose identity provider has not been configured yet.
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAllVerifier;

#[async_trait]
impl TokenVerifier for DenyAllVerifier {
    async fn verify(&self, _token: &str, _endpoint_id: &str) -> Result<PeerIdentity, P2pError> {
        Err(P2pError::Unauthenticated(
            "no token verifier configured; refusing to establish a peer session".to_owned(),
        ))
    }
}

/// The set of peers this node will talk to.
///
/// Discovery is LAN + explicit pairing, not open discovery: finding a peer on
/// the local network does not make it trusted. A peer must be in this set
/// *and* present a verifiable token before a session is established.
#[derive(Debug, Clone, Default)]
pub struct PairingStore {
    paired: BTreeSet<String>,
}

impl PairingStore {
    /// Creates an empty store. An empty store pairs with nobody.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a store pre-populated with known endpoint ids.
    #[must_use]
    pub fn from_endpoint_ids(ids: impl IntoIterator<Item = String>) -> Self {
        Self {
            paired: ids.into_iter().collect(),
        }
    }

    /// Records a peer as paired.
    pub fn pair(&mut self, endpoint_id: impl Into<String>) {
        self.paired.insert(endpoint_id.into());
    }

    /// Removes a pairing. Returns whether the peer had been paired.
    pub fn unpair(&mut self, endpoint_id: &str) -> bool {
        self.paired.remove(endpoint_id)
    }

    /// Returns whether `endpoint_id` is paired with this node.
    #[must_use]
    pub fn is_paired(&self, endpoint_id: &str) -> bool {
        self.paired.contains(endpoint_id)
    }

    /// Returns the number of paired peers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.paired.len()
    }

    /// Returns whether no peers are paired.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.paired.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deny_all_verifier_refuses_every_token() {
        let verifier = DenyAllVerifier;
        let err = verifier
            .verify("any.jwt.here", "endpoint-abc")
            .await
            .expect_err("DenyAllVerifier must never authenticate");
        assert!(matches!(err, P2pError::Unauthenticated(_)));
    }

    #[test]
    fn empty_pairing_store_pairs_with_nobody() {
        let store = PairingStore::new();
        assert!(store.is_empty());
        assert!(!store.is_paired("endpoint-abc"));
    }

    #[test]
    fn pairing_is_explicit_and_reversible() {
        let mut store = PairingStore::new();
        store.pair("endpoint-abc");
        assert!(store.is_paired("endpoint-abc"));
        assert!(!store.is_paired("endpoint-xyz"));

        assert!(store.unpair("endpoint-abc"));
        assert!(!store.is_paired("endpoint-abc"));
        assert!(!store.unpair("endpoint-abc"));
    }
}
