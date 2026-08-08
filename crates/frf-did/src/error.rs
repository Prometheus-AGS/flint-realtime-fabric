use thiserror::Error;

/// Errors raised while deriving or resolving decentralized identifiers.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DidError {
    /// The identifier does not use a DID method this crate supports.
    #[error("unsupported DID method: {0}")]
    UnsupportedMethod(String),

    /// The identifier is structurally invalid.
    #[error("malformed DID: {0}")]
    Malformed(String),

    /// The key type encoded in the DID is not one this crate supports.
    ///
    /// `did:key` can carry secp256k1, P-256, RSA and others. The fabric's peer
    /// transport is Ed25519-only because that is what iroh proves during the
    /// QUIC handshake, so any other key type is rejected rather than silently
    /// accepted as unverifiable.
    #[error("unsupported key type: {0}")]
    UnsupportedKeyType(String),

    /// A `did:web` document could not be retrieved.
    #[error("failed to resolve {did}: {reason}")]
    Resolution { did: String, reason: String },
}
