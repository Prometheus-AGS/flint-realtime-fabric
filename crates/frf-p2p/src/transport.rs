use std::sync::Arc;

use iroh::endpoint::{Connection, presets};
use iroh::{Endpoint, EndpointAddr, EndpointId, SecretKey};

use crate::config::{FRF_P2P_ALPN, PeerConfig};
use crate::error::P2pError;
use crate::identity::{DenyAllVerifier, PairingStore, PeerIdentity, TokenVerifier};

/// A peer-to-peer endpoint that can dial and accept authenticated sessions.
///
/// The endpoint's identity *is* its Ed25519 public key: iroh proves possession
/// of the corresponding secret during the QUIC handshake, so
/// [`PeerSession::endpoint_id`] is the one property a remote peer cannot forge.
/// Everything else a peer asserts is verified by the configured
/// [`TokenVerifier`] before a session is handed back.
pub struct PeerTransport {
    pub(crate) endpoint: Endpoint,
    pub(crate) verifier: Arc<dyn TokenVerifier>,
    pub(crate) pairing: std::sync::RwLock<PairingStore>,
}

/// An established, authenticated session with a peer.
///
/// Holding one of these is proof that the peer's key was verified by the QUIC
/// handshake, that it is paired with this node, and that its presented
/// credential satisfied the configured verifier.
pub struct PeerSession {
    connection: Connection,
    identity: PeerIdentity,
    credential: String,
    verifier: Arc<dyn TokenVerifier>,
}

impl PeerSession {
    /// The verified identity of the remote peer.
    #[must_use]
    pub fn identity(&self) -> &PeerIdentity {
        &self.identity
    }

    /// The peer's endpoint id, proven by the QUIC handshake.
    #[must_use]
    pub fn endpoint_id(&self) -> &str {
        &self.identity.endpoint_id
    }

    /// Revalidate the original peer credential against the host's current authority.
    pub async fn revalidate(&self) -> Result<(), P2pError> {
        match self
            .verifier
            .verify(&self.credential, self.endpoint_id())
            .await
        {
            Ok(identity) if identity == self.identity => Ok(()),
            _ => {
                self.connection
                    .close(2u8.into(), b"current peer authority denied");
                Err(P2pError::Unauthenticated(
                    "current peer authority denied".into(),
                ))
            }
        }
    }

    /// The underlying QUIC connection, for opening streams.
    #[must_use]
    pub fn connection(&self) -> &Connection {
        &self.connection
    }
}

impl PeerTransport {
    /// Binds a peer endpoint using `config`.
    ///
    /// The endpoint is bound but **cannot establish any session** until a
    /// verifier is supplied via [`PeerTransport::with_verifier`]: the default
    /// is [`DenyAllVerifier`]. This is the fail-closed posture that replaces
    /// the compile-time gate — peer support ships enabled, so refusal has to be
    /// enforced at runtime instead.
    ///
    /// # Errors
    ///
    /// Returns [`P2pError::Bind`] if the endpoint cannot be bound, and
    /// [`P2pError::Bind`] if `config.enabled` is `false` — callers should check
    /// [`PeerConfig::enabled`] before calling rather than relying on the error.
    pub async fn bind(
        config: &PeerConfig,
        secret_key: Option<SecretKey>,
    ) -> Result<Self, P2pError> {
        if !config.enabled {
            return Err(P2pError::Bind(
                "peer operation is disabled by configuration".to_owned(),
            ));
        }

        let mut builder = Endpoint::builder(presets::N0).alpns(vec![FRF_P2P_ALPN.to_vec()]);
        if !config.relay_enabled {
            builder = builder.relay_mode(iroh::RelayMode::Disabled);
        }
        if let Some(key) = secret_key {
            builder = builder.secret_key(key);
        }

        let endpoint = builder
            .bind()
            .await
            .map_err(|e| P2pError::Bind(e.to_string()))?;

        Ok(Self {
            endpoint,
            verifier: Arc::new(DenyAllVerifier),
            pairing: std::sync::RwLock::new(PairingStore::from_endpoint_ids(
                config.paired_endpoints.clone(),
            )),
        })
    }

    /// Installs the verifier used to authenticate peers.
    ///
    /// Until this is called, every session attempt is refused.
    #[must_use]
    pub fn with_verifier(mut self, verifier: Arc<dyn TokenVerifier>) -> Self {
        self.verifier = verifier;
        self
    }

    pub(crate) fn is_paired(&self, endpoint: &str) -> Result<bool, P2pError> {
        Ok(self
            .pairing
            .read()
            .map_err(|_| P2pError::Unauthenticated("pairing state unavailable".into()))?
            .is_paired(endpoint))
    }

    /// Update the discovery prerequisite after the host commits explicit consent.
    /// The verifier remains authoritative; pairing alone grants no access.
    pub fn set_paired(&self, endpoint: EndpointId, paired: bool) -> Result<(), P2pError> {
        let mut store = self
            .pairing
            .write()
            .map_err(|_| P2pError::Unauthenticated("pairing state unavailable".into()))?;
        if paired {
            store.pair(endpoint.to_string());
        } else {
            store.unpair(&endpoint.to_string());
        }
        Ok(())
    }

    /// This node's endpoint id — its Ed25519 public key.
    ///
    /// `frf-did` turns this into a `did:key` with no additional key material.
    #[must_use]
    pub fn endpoint_id(&self) -> EndpointId {
        self.endpoint.id()
    }

    /// Dials `addr` and establishes an authenticated session.
    ///
    /// `token` is the credential this node presents to the remote peer.
    ///
    /// # Errors
    ///
    /// - [`P2pError::NotPaired`] if the remote is not in the pairing store.
    ///   Checked **before** dialling, so an unpaired peer costs no round trip.
    /// - [`P2pError::Connect`] if the QUIC connection cannot be established.
    /// - [`P2pError::Unauthenticated`] if the peer's credential does not verify.
    pub async fn connect(
        &self,
        addr: impl Into<EndpointAddr>,
        token: &str,
    ) -> Result<PeerSession, P2pError> {
        let addr = addr.into();
        let remote_id = addr.id.to_string();

        if !self.is_paired(&remote_id)? {
            return Err(P2pError::NotPaired(remote_id));
        }

        let connection = self
            .endpoint
            .connect(addr, FRF_P2P_ALPN)
            .await
            .map_err(|e| P2pError::Connect(e.to_string()))?;

        let (identity, credential) = self.authenticate(&connection, token, false).await?;
        Ok(PeerSession {
            connection,
            identity,
            credential,
            verifier: self.verifier.clone(),
        })
    }

    /// Accept one incoming connection and complete mutual credential admission.
    /// The caller owns the accept loop; this transport creates no scheduler.
    pub async fn accept(&self, token: &str) -> Result<PeerSession, P2pError> {
        let incoming = self
            .endpoint
            .accept()
            .await
            .ok_or_else(|| P2pError::Connect("endpoint closed".into()))?;
        let connection = tokio::time::timeout(std::time::Duration::from_secs(15), incoming)
            .await
            .map_err(|_| P2pError::Connect("QUIC admission deadline exceeded".into()))?
            .map_err(|e| P2pError::Connect(e.to_string()))?;
        let (identity, credential) = self.authenticate(&connection, token, true).await?;
        Ok(PeerSession {
            connection,
            identity,
            credential,
            verifier: self.verifier.clone(),
        })
    }

    async fn authenticate(
        &self,
        connection: &Connection,
        token: &str,
        incoming: bool,
    ) -> Result<(PeerIdentity, String), P2pError> {
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            self.admit(connection, token, incoming),
        )
        .await
        .map_err(|_| P2pError::Unauthenticated("credential admission deadline exceeded".into()))
        .and_then(std::convert::identity);
        if result.is_err() {
            connection.close(1u8.into(), b"admission denied");
        }
        result
    }

    /// Address suitable for explicit out-of-band pairing, never a trust assertion.
    #[must_use]
    pub fn address(&self) -> EndpointAddr {
        self.endpoint.addr()
    }

    /// Closes the endpoint.
    pub async fn close(&self) {
        self.endpoint.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn bind_refuses_when_disabled() {
        let config = PeerConfig {
            enabled: false,
            ..PeerConfig::default()
        };
        let err = PeerTransport::bind(&config, None)
            .await
            .err()
            .expect("binding must fail when disabled");
        assert!(matches!(err, P2pError::Bind(_)));
    }

    #[tokio::test]
    async fn default_transport_cannot_establish_a_session() {
        // The guarantee that replaces the compile-time feature gate: peer
        // support is ON by default, so a node with no verifier configured must
        // still be unable to authenticate anyone.
        let transport = PeerTransport::bind(&PeerConfig::default(), None)
            .await
            .expect("bind with default config");

        let unknown = SecretKey::generate().public();
        let err = transport
            .connect(EndpointAddr::new(unknown), "any.token")
            .await
            .err()
            .expect("a default transport must not establish sessions");

        // Refused at the pairing gate before any network round trip.
        assert!(matches!(err, P2pError::NotPaired(_)));
        transport.close().await;
    }

    #[tokio::test]
    async fn endpoint_id_is_stable_for_a_given_secret_key() {
        // frf-did derives did:key from this value, so it must be a pure
        // function of the secret key rather than per-bind randomness.
        let key = SecretKey::generate();
        let expected = key.public();

        let transport = PeerTransport::bind(&PeerConfig::default(), Some(key))
            .await
            .expect("bind with explicit key");
        assert_eq!(transport.endpoint_id(), expected);
        transport.close().await;
    }
}
