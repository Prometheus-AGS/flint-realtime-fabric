use thiserror::Error;

/// Errors raised by the peer-to-peer transport.
///
/// `Unauthenticated` is deliberately distinct from `Connect`: a peer that is
/// reachable but cannot prove its identity is a *security* outcome, not a
/// network one, and callers are expected to treat the two differently.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum P2pError {
    /// Binding the local endpoint failed.
    #[error("failed to bind endpoint: {0}")]
    Bind(String),

    /// Dialling a remote peer failed before a session was established.
    #[error("failed to connect to peer: {0}")]
    Connect(String),

    /// Opening or accepting a stream on an established connection failed.
    #[error("stream error: {0}")]
    Stream(String),

    /// The peer's identity could not be verified, so no session was established.
    ///
    /// This is the fail-closed path. It is returned both when a verifier
    /// rejects a token and when **no verifier is configured at all** — an
    /// unverifiable peer and an unverified peer are treated identically.
    #[error("peer identity could not be verified: {0}")]
    Unauthenticated(String),

    /// The peer authenticated but is not paired with this node.
    #[error("peer {0} is not paired with this node")]
    NotPaired(String),

    /// The peer authenticated into a different tenant than this node's.
    #[error("peer belongs to tenant {peer}, expected {expected}")]
    TenantMismatch { peer: String, expected: String },

    /// A frame could not be encoded or decoded.
    #[error("codec error: {0}")]
    Codec(String),
}
