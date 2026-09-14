//! `ElectricSQL` shape-facade adapter — the authorized relational read path (ADR-009).
//!
//! ASO clients never reach Electric directly. This crate implements the [`ShapeFacade`] port
//! over a live Electric server. The `frf-app` shape use case owns server-side
//! policy, authorization, and handle-to-grant binding:
//!
//! - **Allowed practice, rows and columns are derived on the server.** Columns come from the
//!   declared policy; the row filter always leads with the server-composed scope predicate.
//! - **Client parameters cannot widen an approved shape.** A key outside the shape's
//!   allow-list is an error, not an ignored field.
//! - **Every request and continuation is authorized.** The application use case
//!   is the only producer of [`frf_ports::AuthorizedShapeRequest`].
//! - **Electric's protocol is preserved.** Snapshot, continuation, handle, offset and refetch
//!   semantics pass through unaltered.
//!
//! This crate implements exactly one port ([`ShapeFacade`]) and holds no authorization
//! dependency of its own. The gateway composes this adapter into `frf-app`.
//!
//! # Status
//!
//! A bounded local composition exercised real Kratos v26.2.0 sessions, Gate RS256 grants,
//! the FRF verifier and Electric 1.8.0 on 2026-09-08. It proved an initial exchange, a
//! same-session continuation with a freshly minted Gate token, expired-handle denial, and
//! rejection of scope, projection and cross-identity handle changes. A client-only Compose
//! segment reached FRF while direct Electric and operator-loopback access failed. The lane is
//! still not certified: persisted row-transition, measured session/membership revocation and
//! deployment-specific topology proofs remain open. The gateway route stays behind the
//! off-by-default `shape-facade` feature.

#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod client;
pub mod error;
pub mod facade;

pub use client::{ElectricUpstream, HttpElectric};
pub use error::ShapeError;
pub use facade::ElectricShapeFacade;
