//! Flint Realtime Fabric — hand-written Rust client SDK.
//!
//! This is the single, canonical Rust client for FRF. It wraps the gateway's
//! `SpineService` (gRPC) with a domain-typed API: [`FrfClient::connect`],
//! [`FrfClient::publish`], [`FrfClient::subscribe`], and [`FrfClient::ack`].
//!
//! Per the project's SDK strategy, business logic — connection lifecycle,
//! reconnection/backoff (p16-c012), and CRDT merge — lives here in exactly one
//! place; other-language SDKs bind to this core rather than reimplementing it.
//!
//! # Example
//!
//! ```no_run
//! use frf_sdk_rust::FrfClient;
//! use frf_domain::{ChannelId, Offset};
//!
//! # async fn run() -> Result<(), frf_sdk_rust::SdkError> {
//! let mut client = FrfClient::connect("http://localhost:9090", None).await?;
//! let mut stream = client
//!     .subscribe(ChannelId::new(), "my-consumer".to_owned(), Offset::BEGINNING)
//!     .await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod convert;
mod error;
mod resilient;
mod services;
#[cfg(feature = "shape-facade")]
mod shape;

pub use client::{AuthInterceptor, FrfClient};
pub use error::SdkError;
pub use resilient::{ReconnectPolicy, SubscribeTarget, resilient_subscribe};
pub use services::ServiceClients;

/// Authorized shape facade client (ADR-009), behind the `shape-facade` feature.
///
/// Off by default, mirroring the gateway's own gate: the lane is uncertified,
/// and a consumer that does not use it should not inherit an HTTP/TLS stack.
#[cfg(feature = "shape-facade")]
pub use shape::{
    ShapeClient, ShapeCursor, ShapeFrame, ShapeMessage, ShapeMessageHeaders, ShapeRequestOptions,
};

#[cfg(all(feature = "shape-facade", not(target_arch = "wasm32")))]
pub use shape::ShapeStreamHead;
