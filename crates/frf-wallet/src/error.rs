use frf_did::DidError;
use thiserror::Error;

/// Errors raised while holding, presenting, or verifying credentials.
///
/// Each variant names a distinct way authorisation can fail. They are kept
/// separate rather than collapsed into one "invalid credential" case because
/// the operator needs to tell a clock-skew problem from a stolen credential.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WalletError {
    /// The proof type is not one this crate can verify.
    #[error("unsupported proof type: {0}")]
    UnsupportedProof(String),

    /// The signature did not verify against the issuer's key.
    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    /// The credential names one issuer but was signed by a different key.
    #[error("credential names issuer {issuer} but was signed by {signer}")]
    IssuerMismatch { issuer: String, signer: String },

    /// The credential was issued by someone other than the expected owner.
    #[error("untrusted issuer: expected {expected}, found {found}")]
    UntrustedIssuer { expected: String, found: String },

    /// The credential was issued to a different node.
    ///
    /// A correctly-signed, unexpired credential presented by a device it was
    /// not issued to is a replay, not a formatting problem.
    #[error("credential was issued to {found}, not {expected}")]
    SubjectMismatch { expected: String, found: String },

    /// The credential is outside its validity window.
    #[error("credential is outside its validity window")]
    Expired,

    /// The credential does not grant the requested capability.
    #[error("capability not granted: {0}")]
    CapabilityNotGranted(String),

    /// A DID in the credential could not be parsed.
    #[error(transparent)]
    Did(#[from] DidError),

    /// The credential could not be serialized for signing or verification.
    #[error("failed to canonicalize credential: {0}")]
    Canonicalization(String),
}
