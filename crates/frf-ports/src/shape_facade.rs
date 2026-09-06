//! The `ShapeFacade` port — authorized relational replication between `ElectricSQL` and an
//! ASO local SQL replica (ADR-009).
//!
//! ASO clients do not talk to Electric directly. They talk to this facade, which derives the
//! allowed practice, rows and columns **on the server**, authorizes every request *and every
//! continuation*, and forwards a constrained request upstream. Client-supplied parameters can
//! narrow an approved shape but never widen it.
//!
//! The facade is a **read path only**. Per ADR-009 the FRF event, CRDT, agent and media lanes
//! are separate and cannot write the same authoritative clinical rows — one feed owns each
//! clinical entity type. Nothing on this port writes.
//!
//! Electric's protocol semantics — snapshot, continuation, handle, offset and refetch — are
//! preserved end to end so the client's local replica stays a faithful Electric consumer. The
//! facade constrains *what* may be requested; it does not reinterpret the protocol.
//!
//! This crate holds no implementation (the absolute dependency rule). The adapter lives in
//! `frf-shape-electric`; authorization is composed over this port in `frf-gateway`, never
//! inside the adapter.

use async_trait::async_trait;
use frf_domain::TenantId;

use crate::error::PortError;

/// Where a client is resuming an Electric stream from.
///
/// Mirrors Electric's own cursor: a fresh consumer starts at [`Cursor::Initial`]; a resuming
/// one carries back the `handle` and `offset` it last saw. Both are opaque to the facade —
/// they are Electric's values, echoed, never synthesized. A resuming cursor is **not**
/// evidence of authorization: every continuation is re-authorized (ADR-009).
// Deliberately exhaustive: adapters must handle both arms, and a third cursor state would be
// a protocol change that every adapter has to reason about rather than silently fall through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    /// No prior state — request the full initial snapshot for the shape.
    Initial,
    /// Resume an existing shape stream at `offset` under `handle`.
    Resume {
        /// Electric's shape handle, as previously returned upstream.
        handle: String,
        /// Electric's offset within that handle.
        offset: String,
    },
}

/// A client's request for one shape, before authorization.
///
/// `shape` names a shape **declared in server-side policy** — not a table, and not a
/// client-authored query. `params` may only narrow within that declaration; the facade
/// rejects any key the policy does not allow, rather than ignoring it, so a widening attempt
/// fails loudly instead of silently returning a broader set.
#[derive(Debug, Clone)]
pub struct ShapeRequest {
    /// Tenant the request is made within.
    pub tenant_id: TenantId,
    /// Authenticated subject (server-assigned; never a client-supplied identity).
    pub subject: String,
    /// Policy-declared shape id.
    pub shape: String,
    /// Client-supplied narrowing parameters, validated against policy.
    pub params: Vec<(String, String)>,
    /// Where to resume from.
    pub cursor: Cursor,
}

/// One chunk of Electric's response, with its protocol metadata preserved.
///
/// `body` is Electric's own payload, forwarded unaltered — the facade authorizes and
/// constrains requests, it does not rewrite response data. `must_refetch` surfaces Electric's
/// refetch signal so the client can rebuild the affected replica generation rather than
/// merging a new snapshot into stale rows (ADR-009, §8 of the ASO runtime architecture).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ShapeChunk {
    /// Electric's shape handle for this stream.
    pub handle: String,
    /// The offset a client should resume from next.
    pub next_offset: String,
    /// Electric's response payload, unmodified.
    pub body: Vec<u8>,
    /// `true` when upstream signalled that the client must discard and refetch.
    pub must_refetch: bool,
    /// `true` when this chunk completes the initial snapshot — the client may not present a
    /// cold-start view as current until it has seen this.
    pub snapshot_complete: bool,
}

impl ShapeChunk {
    /// Build a chunk from an upstream response.
    ///
    /// `ShapeChunk` is `#[non_exhaustive]` so new protocol metadata can be added without
    /// breaking downstream matches; adapters construct it through this constructor rather
    /// than a struct literal.
    #[must_use]
    pub const fn new(
        handle: String,
        next_offset: String,
        body: Vec<u8>,
        must_refetch: bool,
        snapshot_complete: bool,
    ) -> Self {
        Self {
            handle,
            next_offset,
            body,
            must_refetch,
            snapshot_complete,
        }
    }
}

/// An authorized, server-derived relational read path over `ElectricSQL`.
///
/// Implemented by `frf-shape-electric`. Adapter methods MUST be instrumented with
/// `#[tracing::instrument]`, and MUST NOT log rows, tokens or clinical text — replica
/// diagnostics are limited to counts, durations, schema versions, anonymized correlation and
/// error classes (ASO runtime architecture §12).
///
/// # Authorization
///
/// Implementations of this port are **not** the authorization boundary; they receive a
/// request that the gateway has already resolved and authorized. This mirrors ADR-007's
/// media-path split: the adapter stays a pure transport, and composition of
/// (shape × authz) happens only in `frf-gateway`.
#[async_trait]
pub trait ShapeFacade: Send + Sync + 'static {
    /// Fetch the next chunk for an already-authorized, already-constrained request.
    ///
    /// # Errors
    ///
    /// [`PortError::NotFound`] if the shape is unknown upstream, [`PortError::Timeout`] on a
    /// slow upstream, [`PortError::Transport`] or [`PortError::Upstream`] on a failed
    /// exchange.
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeChunk, PortError>;
}

/// A [`ShapeRequest`] that has passed policy resolution and an authorization check.
///
/// This type is the seam that makes "authorize every continuation" structural rather than a
/// convention: the adapter's `fetch` takes only this, and it cannot be constructed outside
/// the gateway's authorization path (its fields are populated by the resolver, which is the
/// only place the Keto check happens). An unauthorized request is therefore not merely
/// rejected — it is unrepresentable at the transport boundary.
#[derive(Debug, Clone)]
pub struct AuthorizedShapeRequest {
    /// The upstream table this shape reads.
    pub table: String,
    /// The exact column set the subject may see. Never client-supplied.
    pub columns: Vec<String>,
    /// The server-derived row filter, including the subject's practice scoping.
    pub where_clause: String,
    /// Cursor to forward upstream.
    pub cursor: Cursor,
}
