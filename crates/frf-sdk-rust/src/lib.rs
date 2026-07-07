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

pub use client::{AuthInterceptor, FrfClient};
pub use error::SdkError;
pub use resilient::{ReconnectPolicy, SubscribeTarget, resilient_subscribe};
pub use services::ServiceClients;
