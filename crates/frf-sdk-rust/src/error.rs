use thiserror::Error;

/// Errors returned by the Flint Realtime Fabric Rust client.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SdkError {
    /// Failed to establish the transport connection to the gateway.
    #[error("connect failed: {0}")]
    Connect(String),

    /// The gateway rejected a request (gRPC status).
    #[error("request failed: {0}")]
    Request(String),

    /// A value returned by the gateway could not be converted to a domain type.
    #[error("invalid response: {0}")]
    InvalidResponse(String),

    /// The provided gateway endpoint URL was not valid.
    #[error("invalid endpoint: {0}")]
    InvalidEndpoint(String),

    // ── Authorized shape facade (ADR-009) ───────────────────────────────────
    //
    // These variants are present in every build, not gated behind
    // `shape-facade`. A feature that changes the *shape* of a public enum makes
    // `SdkError` mean different things in different builds, which is worse for
    // a consumer than a few variants they never construct. Only the module and
    // its HTTP dependency are gated.
    /// `401` — the grant backing the request expired.
    ///
    /// Recoverable: re-establish the session and retry. Not a transient
    /// network fault, so a backoff loop that assumes one will spin forever —
    /// the credential itself has to change.
    #[error("shape grant expired: the session must be re-established")]
    ShapeGrantExpired,

    /// `403` — authorization refused, or the continuation handle is not this
    /// grant's.
    ///
    /// The facade returns 403 for both `Unauthorized` and `HandleMismatch` and
    /// deliberately does not distinguish them on the wire: saying which would
    /// disclose whether a handle exists.
    #[error("shape access forbidden: the grant does not cover this request")]
    ShapeForbidden,

    /// `404` — the shape id is not in the server's catalog.
    ///
    /// **Ambiguous by construction.** A gateway built without
    /// `--features shape-facade` does not mount `/v1/shape` at all and returns
    /// a router 404, which is indistinguishable from a genuine catalog miss.
    /// Check the gateway's build flags before concluding the id is wrong.
    #[error(
        "unknown shape '{0}': not in the server catalog, or the gateway lacks --features shape-facade"
    )]
    UnknownShape(String),

    /// `400` — the request was malformed.
    ///
    /// Two causes in practice: a half-formed cursor (a `handle` without an
    /// `offset`, which the facade refuses rather than guessing at), or a
    /// narrowing parameter the shape's `allowed_params` does not permit.
    #[error("invalid shape request: {0}")]
    InvalidShapeRequest(String),

    /// `5xx` — the facade reached Electric and the exchange failed upstream.
    ///
    /// Unlike every refusal above, nothing about the grant is wrong, so this
    /// *is* a candidate for retry with backoff. Covers 502 (upstream failure),
    /// 503 (clock before the Unix epoch) and 500 (unrecognized failure).
    #[error("shape upstream failed: {0}")]
    ShapeUpstream(String),
}

impl From<tonic::Status> for SdkError {
    fn from(status: tonic::Status) -> Self {
        SdkError::Request(format!("{}: {}", status.code(), status.message()))
    }
}
