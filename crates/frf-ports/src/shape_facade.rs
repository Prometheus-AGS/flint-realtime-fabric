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
//! `frf-shape-electric`; `frf-app` composes authorization and server policy over this port,
//! and `frf-gateway` supplies the concrete dependencies.

use std::pin::Pin;

use async_trait::async_trait;
use bytes::Bytes;
use frf_domain::TenantId;
use futures_core::Stream;

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
    /// Start at Electric's current log position without historical rows.
    Now,
    /// Resume an existing shape stream at `offset` under `handle`.
    Resume {
        /// Electric's shape handle, as previously returned upstream.
        handle: String,
        /// Electric's offset within that handle.
        offset: String,
    },
}

/// Electric protocol options that a trusted facade may forward unchanged.
///
/// Shape-defining values (`table`, `where`, and `columns`) deliberately do not
/// appear here. The server derives those from policy after authorization.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShapeProtocol {
    /// Electric long-polling mode from the `live` query parameter.
    pub live: Option<String>,
    /// Electric cache-busting cursor from the `cursor` query parameter.
    pub cursor: Option<String>,
    /// Conditional request validator from `If-None-Match`.
    pub if_none_match: Option<String>,
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
    /// Client protocol options that are safe to forward unchanged.
    pub protocol: ShapeProtocol,
}

/// One HTTP response header preserved from Electric.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeHeader {
    /// Lowercase HTTP header name.
    pub name: String,
    /// Header bytes as received from Electric.
    pub value: Vec<u8>,
}

impl ShapeHeader {
    /// Construct a preserved response header.
    #[must_use]
    pub fn new(name: impl Into<String>, value: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Shell-neutral stream of protected Electric response frames.
///
/// The Electric adapter supplies frames in upstream order. The application layer wraps this
/// stream with a [`ShapeLease`] before it crosses the interface boundary. Dropping the wrapped
/// stream must release its lease and any uncommitted continuation state. Axum, Hyper, Tauri, or
/// another shell type must not appear in this contract.
pub type ShapeBodyStream = Pin<Box<dyn Stream<Item = Result<Bytes, PortError>> + Send + 'static>>;

/// Authority decision bound to one protected shape response.
///
/// One lease instance captures one verified grant and its server-derived authorization tuple.
/// [`ShapeLease::revalidate`] returns `Ok(())` only while that exact authority remains current.
/// Permission denial, expiry, timeout, transport loss, and every other error are terminal for the
/// protected body that owns the lease; callers must cancel rather than reuse an earlier success.
/// The application layer owns the monotonic cancellation worker and the response stream owns that
/// worker through completion or drop.
#[async_trait]
pub trait ShapeLease: Send + Sync + 'static {
    /// Revalidate the exact grant captured for this response.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::PermissionDenied`] when authority has ended and another
    /// [`PortError`] when current authority cannot be established. Both outcomes require the
    /// protected stream to terminate without another frame.
    async fn revalidate(&self) -> Result<(), PortError>;
}

/// One Electric HTTP response, with protocol metadata and control frames intact.
///
/// The adapter does not translate response status, headers, or body. This keeps
/// initial snapshots, 304 responses, 409 must-refetch frames, schema metadata,
/// cursors, cache validators, and future control messages observable to the real
/// Electric client.
#[non_exhaustive]
pub struct ShapeResponse {
    /// Exact upstream HTTP status code.
    pub status: u16,
    /// Electric protocol and HTTP cache headers selected by the adapter.
    pub headers: Vec<ShapeHeader>,
    /// Electric response frames, including control messages, in upstream order.
    pub body: ShapeBodyStream,
}

impl ShapeResponse {
    /// Build a preserved response from one buffered body.
    ///
    /// `ShapeResponse` is `#[non_exhaustive]` so new protocol metadata can be added without
    /// breaking downstream matches; adapters construct it through this constructor rather
    /// than a struct literal.
    #[must_use]
    pub fn new(status: u16, headers: Vec<ShapeHeader>, body: Vec<u8>) -> Self {
        let frame = (!body.is_empty()).then(|| Ok(Bytes::from(body)));
        Self::streamed(status, headers, Box::pin(futures_util::stream::iter(frame)))
    }

    /// Build a preserved response from an ordered body stream.
    #[must_use]
    pub fn streamed(status: u16, headers: Vec<ShapeHeader>, body: ShapeBodyStream) -> Self {
        Self {
            status,
            headers,
            body,
        }
    }

    /// Return a preserved header using an ASCII case-insensitive lookup.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&[u8]> {
        self.headers
            .iter()
            .find(|header| header.name.eq_ignore_ascii_case(name))
            .map(|header| header.value.as_slice())
    }
}

impl std::fmt::Debug for ShapeResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ShapeResponse")
            .field("status", &self.status)
            .field("headers", &self.headers)
            .field("body", &"<stream>")
            .finish_non_exhaustive()
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
/// request that the application use case has already resolved and authorized. The adapter
/// stays a pure transport; `frf-app` owns the use case and the gateway is its composition
/// root.
#[async_trait]
pub trait ShapeFacade: Send + Sync + 'static {
    /// Fetch the next chunk for an already-authorized, already-constrained request.
    ///
    /// # Errors
    ///
    /// [`PortError::NotFound`] if the shape is unknown upstream, [`PortError::Timeout`] on a
    /// slow upstream, [`PortError::Transport`] or [`PortError::Upstream`] on a failed
    /// exchange.
    async fn fetch(&self, request: &AuthorizedShapeRequest) -> Result<ShapeResponse, PortError>;
}

/// A [`ShapeRequest`] that has passed policy resolution and an authorization check.
///
/// This type is the seam that makes "authorize every continuation" structural rather than a
/// convention: the adapter's `fetch` takes only this, and the application use case constructs
/// it only after server policy and the live authorization check pass. An unauthorized request
/// is therefore unrepresentable at the transport boundary.
#[derive(Debug, Clone)]
pub struct AuthorizedShapeRequest {
    /// The upstream table this shape reads.
    pub table: String,
    /// The exact column set the subject may see. Never client-supplied.
    pub columns: Vec<String>,
    /// The server-derived row filter. Practice projections always carry one;
    /// explicitly approved reference projections may omit it.
    pub where_clause: Option<String>,
    /// Cursor to forward upstream.
    pub cursor: Cursor,
    /// Authorized Electric protocol options to forward unchanged.
    pub protocol: ShapeProtocol,
}
