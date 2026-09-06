//! `ElectricSQL` shape-facade adapter — the authorized relational read path (ADR-009).
//!
//! ASO clients never reach Electric directly. This crate implements the [`ShapeFacade`] port
//! over a live Electric server, and supplies the server-side policy and resolution that make
//! ADR-009's guarantees structural rather than conventional:
//!
//! - **Allowed practice, rows and columns are derived on the server.** Columns come from the
//!   declared policy; the row filter always leads with the server-composed scope predicate.
//! - **Client parameters cannot widen an approved shape.** A key outside the shape's
//!   allow-list is an error, not an ignored field.
//! - **Every request *and every continuation* is authorized.** [`ShapeResolver::resolve`] is
//!   the only way to obtain an [`frf_ports::AuthorizedShapeRequest`], and it checks on every
//!   call — a resumed cursor carries no standing permission.
//! - **Electric's protocol is preserved.** Snapshot, continuation, handle, offset and refetch
//!   semantics pass through unaltered.
//!
//! This crate implements exactly one port ([`ShapeFacade`]) and holds no authorization
//! dependency of its own: the resolver takes an [`frf_ports::AuthzProvider`] injected by the
//! gateway, which is the sole composition point (the absolute dependency rule; ADR-007's
//! split).
//!
//! # Status
//!
//! The policy, resolution and protocol-mapping logic are unit-tested. The **live Electric
//! exchange is not verified against a running server** — no Electric deployment was
//! exercised, and ASO has not yet defined the privacy-approved replica schema (sequence
//! step 1 of the ASO runtime architecture). Per ADR-009 this lane stays disabled until its
//! verification criteria are proved; the gateway route is behind the off-by-default
//! `shape-facade` feature.

#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod error;
pub mod facade;
pub mod policy;
pub mod resolver;

pub use client::{ElectricUpstream, HttpElectric, UpstreamResponse};
pub use error::ShapeError;
pub use facade::ElectricShapeFacade;
pub use policy::{ShapeCatalog, ShapePolicy};
pub use resolver::ShapeResolver;
