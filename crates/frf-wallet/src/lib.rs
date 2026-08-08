#![deny(warnings)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

//! Verifiable credential wallet for Flint Realtime Fabric.
//!
//! Implements owner-to-node delegation: an owner (a person or an organisation)
//! issues a [`credential::Credential`] to a node's DID granting specific
//! [`credential::Capability`] values, and the node presents it when pairing.
//!
//! # Why not a shared organisation key
//!
//! The obvious alternative — put one organisation credential on every node — has
//! no per-node revocation, and a single compromised device compromises the whole
//! estate. Per-node delegation means a lost laptop costs exactly one credential.
//!
//! # No central authority
//!
//! Issuance, holding, and verification all use Ed25519 keys the participants
//! already own, and nothing here performs I/O. A node with no network uplink can
//! still verify that a peer was authorised by an owner it trusts. That is what
//! makes this usable on a machine sitting in someone's house.
//!
//! The signing key is the *same* key that authenticates the QUIC session
//! (`iroh::SecretKey`), so there is no second key hierarchy to manage and no way
//! for a credential key and a transport key to drift apart.
//!
//! # The authorisation check
//!
//! [`SignedCredential::verify_delegation`] is the only method that answers
//! "may this node do this?". It checks, in order: the signature, that the signer
//! is the stated issuer, that the issuer is the owner the verifier expects, that
//! the subject is *this* node, that the credential is unexpired, and that the
//! capability was actually granted.
//!
//! Those checks are bundled deliberately. [`SignedCredential::verify_signature`]
//! is public because callers need it when storing a credential, but a valid
//! signature alone authorises nothing — a correctly-signed credential issued to
//! a different device is a replay, and checking the signature without the
//! subject binding would accept it.

pub mod credential;
pub mod error;
pub mod proof;
pub mod wallet;

pub use credential::{
    Capability, Credential, CredentialSubject, NODE_DELEGATION_TYPE, VC_CONTEXT_V2,
};
pub use error::WalletError;
pub use proof::{PROOF_TYPE_ED25519, Proof, SignedCredential, sign_credential};
pub use wallet::Wallet;
