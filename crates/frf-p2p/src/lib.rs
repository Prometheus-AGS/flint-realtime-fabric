#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Peer-to-peer transport for Flint Realtime Fabric.
//!
//! Establishes direct, authenticated device-to-device sessions over
//! [iroh](https://iroh.computer) QUIC — no intermediate server, no listening
//! port to bind, and no proprietary protocol.
//!
//! # Why this exists
//!
//! A device that cannot bind a listening port is not thereby unreachable.
//! Binding a port and *being addressable* are different things: with a P2P
//! transport the device dials out, establishes an authenticated session, and
//! serves requests over it. That is what makes a phone or a laptop a first-class
//! participant rather than a client.
//!
//! # Identity
//!
//! An iroh endpoint's id **is** its Ed25519 public key
//! (`iroh_base::EndpointId` is a type alias for `PublicKey`). The QUIC handshake
//! proves possession of the matching secret, so the peer's key is the one thing
//! it cannot lie about. `frf-did` re-encodes those same 32 bytes as a `did:key`,
//! which is why decentralized identity costs no additional key material here.
//!
//! # Security posture
//!
//! Peer support is **compiled into every build and enabled by default**; it is
//! disabled through configuration, never a Cargo feature. Because there is no
//! build-time gate standing between a default build and peer reachability, the
//! runtime must fail closed:
//!
//! - The default verifier is [`identity::DenyAllVerifier`], which authenticates
//!   nobody. A node with no configured verifier can accept no sessions.
//! - Discovery never implies trust: a peer must be in the [`identity::PairingStore`]
//!   **and** present a credential that verifies.
//!
//! Default-on does not mean auth-optional.

pub mod config;
pub mod error;
pub mod identity;
pub mod transport;

pub use config::{FRF_P2P_ALPN, PeerConfig};
pub use error::P2pError;
pub use identity::{DenyAllVerifier, PairingStore, PeerIdentity, TokenVerifier};
pub use transport::{PeerSession, PeerTransport};
