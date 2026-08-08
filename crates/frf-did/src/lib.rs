#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Decentralized identifiers for Flint Realtime Fabric.
//!
//! # The key idea
//!
//! An iroh endpoint's id **is** an Ed25519 public key, and a `did:key` is that
//! same key under a multicodec prefix and base58btc encoding. So a node's DID is
//! a *pure function* of the key it already owns:
//!
//! ```text
//!   iroh SecretKey ──► EndpointId (32-byte Ed25519 public key)
//!                          │
//!                          ├──► QUIC handshake proves possession
//!                          │
//!                          └──► did:key:z6Mk…  (0xed01 ‖ key, base58btc)
//! ```
//!
//! Decentralized identity therefore costs no extra key material, no registry,
//! and no network access. [`did_key_from_endpoint`] is deterministic and works
//! offline — which is what lets a node in a house with no uplink still
//! authenticate a peer.
//!
//! # A DID is a claim until it is checked
//!
//! The handshake proves the *key*, not the DID a peer asserts. Always call
//! [`did_matches_endpoint`] before trusting an asserted DID; a mismatch is an
//! impersonation attempt, not a formatting problem.
//!
//! # Choosing a method
//!
//! | | `did:key` | `did:web` |
//! |---|---|---|
//! | Infrastructure | none | a domain and HTTPS |
//! | Works offline | **yes** | no |
//! | **Key rotation** | **no — ever** | yes |
//!
//! `did:key` is the default because it fits a device someone owns personally.
//! Its inability to rotate is a genuine limitation, not a footnote: if the key
//! leaks, the identity is dead and must be re-paired everywhere. Nodes that
//! control a domain should upgrade to [`did_web_document_url`]-backed
//! identifiers, accepting that the identity is then only as decentralized as
//! the domain.

pub mod error;
pub mod key;
pub mod web;

pub use error::DidError;
pub use key::{
    did_key_from_bytes, did_key_from_endpoint, did_matches_endpoint, public_key_from_did_key,
};
pub use web::did_web_document_url;
