use chrono::{DateTime, Utc};
use frf_did::{did_key_from_endpoint, public_key_from_did_key};
use iroh::{PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};

use crate::credential::{Capability, Credential};
use crate::error::WalletError;

/// A credential together with the issuer's Ed25519 signature over it.
///
/// The signature covers the canonical JSON serialization of the credential, so
/// any change to the issuer, subject, capabilities, or validity window
/// invalidates it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedCredential {
    #[serde(flatten)]
    pub credential: Credential,
    pub proof: Proof,
}

/// An Ed25519 proof over a credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    #[serde(rename = "type")]
    pub proof_type: String,
    /// The DID whose key produced this signature.
    #[serde(rename = "verificationMethod")]
    pub verification_method: String,
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: String,
    /// base58btc-encoded Ed25519 signature.
    #[serde(rename = "proofValue")]
    pub proof_value: String,
}

/// Proof type identifier for the Ed25519 signatures produced here.
pub const PROOF_TYPE_ED25519: &str = "Ed25519Signature2020";

/// Signs `credential` with `issuer_key`.
///
/// The issuer's `did:key` is derived from the signing key itself, so the
/// `verificationMethod` cannot disagree with the key that actually signed.
///
/// # Errors
///
/// Returns [`WalletError::Canonicalization`] if the credential cannot be
/// serialized.
pub fn sign_credential(
    credential: Credential,
    issuer_key: &SecretKey,
) -> Result<SignedCredential, WalletError> {
    let issuer_did = did_key_from_endpoint(&issuer_key.public());
    let payload = canonical_bytes(&credential)?;
    let signature = issuer_key.sign(&payload);

    Ok(SignedCredential {
        credential,
        proof: Proof {
            proof_type: PROOF_TYPE_ED25519.to_owned(),
            verification_method: issuer_did,
            proof_purpose: "assertionMethod".to_owned(),
            proof_value: bs58::encode(signature.to_bytes()).into_string(),
        },
    })
}

impl SignedCredential {
    /// Verifies the signature and the credential's internal consistency.
    ///
    /// This checks that the signature is valid **and** that the key which
    /// produced it is the one named by the credential's `issuer`. Without the
    /// second check, anyone could sign a credential that names someone else as
    /// issuer and it would still verify.
    ///
    /// Validity dates and capability grants are **not** checked here — see
    /// [`SignedCredential::verify_delegation`], which checks everything a peer
    /// actually needs before acting.
    ///
    /// # Errors
    ///
    /// - [`WalletError::UnsupportedProof`] if the proof type is not Ed25519.
    /// - [`WalletError::IssuerMismatch`] if the signing key is not the issuer.
    /// - [`WalletError::InvalidSignature`] if the signature does not verify.
    pub fn verify_signature(&self) -> Result<(), WalletError> {
        if self.proof.proof_type != PROOF_TYPE_ED25519 {
            return Err(WalletError::UnsupportedProof(self.proof.proof_type.clone()));
        }

        // The signer must be the credential's stated issuer.
        if self.proof.verification_method != self.credential.issuer {
            return Err(WalletError::IssuerMismatch {
                issuer: self.credential.issuer.clone(),
                signer: self.proof.verification_method.clone(),
            });
        }

        let key_bytes = public_key_from_did_key(&self.proof.verification_method)?;
        let public_key = PublicKey::from_bytes(&key_bytes)
            .map_err(|e| WalletError::InvalidSignature(e.to_string()))?;

        let raw = bs58::decode(&self.proof.proof_value)
            .into_vec()
            .map_err(|e| WalletError::InvalidSignature(format!("base58 decode failed: {e}")))?;
        let raw: [u8; 64] = raw.as_slice().try_into().map_err(|_| {
            WalletError::InvalidSignature(format!(
                "expected a 64-byte signature, found {}",
                raw.len()
            ))
        })?;

        let payload = canonical_bytes(&self.credential)?;
        public_key
            .verify(&payload, &Signature::from_bytes(&raw))
            .map_err(|e| WalletError::InvalidSignature(e.to_string()))
    }

    /// Verifies that this credential authorises `node_key` to exercise
    /// `capability` at `now`, on behalf of `expected_issuer_did`.
    ///
    /// This is the check a peer performs before acting on a request. It is
    /// deliberately the *only* public entry point that returns a positive
    /// authorisation answer, so no caller can accidentally check the signature
    /// and forget the subject binding, the expiry, or the grant.
    ///
    /// # Errors
    ///
    /// - Any error from [`SignedCredential::verify_signature`].
    /// - [`WalletError::UntrustedIssuer`] if the issuer is not the expected owner.
    /// - [`WalletError::SubjectMismatch`] if the credential was issued to a
    ///   different node — this is the stolen-credential case.
    /// - [`WalletError::Expired`] if outside the validity window.
    /// - [`WalletError::CapabilityNotGranted`] if the capability was not delegated.
    pub fn verify_delegation(
        &self,
        expected_issuer_did: &str,
        node_key: &PublicKey,
        capability: Capability,
        now: DateTime<Utc>,
    ) -> Result<(), WalletError> {
        self.verify_signature()?;

        if self.credential.issuer != expected_issuer_did {
            return Err(WalletError::UntrustedIssuer {
                expected: expected_issuer_did.to_owned(),
                found: self.credential.issuer.clone(),
            });
        }

        // The credential must have been issued to *this* node. A valid
        // credential presented by a different device is a replay.
        let node_did = did_key_from_endpoint(node_key);
        if self.credential.credential_subject.id != node_did {
            return Err(WalletError::SubjectMismatch {
                expected: node_did,
                found: self.credential.credential_subject.id.clone(),
            });
        }

        if !self.credential.is_valid_at(now) {
            return Err(WalletError::Expired);
        }

        if !self.credential.grants(capability) {
            return Err(WalletError::CapabilityNotGranted(format!("{capability:?}")));
        }

        Ok(())
    }
}

/// Serializes a credential deterministically for signing.
///
/// `serde_json` emits struct fields in declaration order and
/// [`serde_json::Map`] preserves insertion order, so the same credential value
/// always produces identical bytes.
fn canonical_bytes(credential: &Credential) -> Result<Vec<u8>, WalletError> {
    serde_json::to_vec(credential).map_err(|e| WalletError::Canonicalization(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::Capability;

    fn at(offset_hours: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + offset_hours * 3600, 0)
            .unwrap_or(DateTime::UNIX_EPOCH)
    }

    struct Fixture {
        owner: SecretKey,
        node: SecretKey,
        signed: SignedCredential,
    }

    fn fixture(capabilities: Vec<Capability>) -> Fixture {
        let owner = SecretKey::generate();
        let node = SecretKey::generate();
        let credential = Credential::delegation(
            did_key_from_endpoint(&owner.public()),
            did_key_from_endpoint(&node.public()),
            capabilities,
            at(0),
            Some(at(24)),
        );
        let signed = sign_credential(credential, &owner).expect("signing succeeds");
        Fixture {
            owner,
            node,
            signed,
        }
    }

    #[test]
    fn an_owner_issued_credential_verifies() {
        let f = fixture(vec![Capability::ExecuteInference]);
        f.signed.verify_signature().expect("signature verifies");

        f.signed
            .verify_delegation(
                &did_key_from_endpoint(&f.owner.public()),
                &f.node.public(),
                Capability::ExecuteInference,
                at(1),
            )
            .expect("delegation authorises the node");
    }

    #[test]
    fn a_credential_from_a_different_issuer_is_rejected() {
        // D-P6's core guarantee: delegation constrains. A credential signed by
        // someone who is not the expected owner must not authorise anything.
        let f = fixture(vec![Capability::ExecuteInference]);
        let stranger = SecretKey::generate();

        let err = f
            .signed
            .verify_delegation(
                &did_key_from_endpoint(&stranger.public()),
                &f.node.public(),
                Capability::ExecuteInference,
                at(1),
            )
            .expect_err("a stranger's DID must not be accepted as issuer");
        assert!(matches!(err, WalletError::UntrustedIssuer { .. }));
    }

    #[test]
    fn a_credential_issued_to_another_node_is_rejected() {
        // The stolen-credential case: a genuine, unexpired, correctly-signed
        // credential presented by a device it was not issued to.
        let f = fixture(vec![Capability::ExecuteInference]);
        let other_node = SecretKey::generate();

        let err = f
            .signed
            .verify_delegation(
                &did_key_from_endpoint(&f.owner.public()),
                &other_node.public(),
                Capability::ExecuteInference,
                at(1),
            )
            .expect_err("a credential must not authorise a different node");
        assert!(matches!(err, WalletError::SubjectMismatch { .. }));
    }

    #[test]
    fn an_ungranted_capability_is_rejected() {
        let f = fixture(vec![Capability::SyncState]);
        let err = f
            .signed
            .verify_delegation(
                &did_key_from_endpoint(&f.owner.public()),
                &f.node.public(),
                Capability::ExecuteInference,
                at(1),
            )
            .expect_err("only delegated capabilities may be exercised");
        assert!(matches!(err, WalletError::CapabilityNotGranted(_)));
    }

    #[test]
    fn an_expired_credential_is_rejected() {
        let f = fixture(vec![Capability::ExecuteInference]);
        let err = f
            .signed
            .verify_delegation(
                &did_key_from_endpoint(&f.owner.public()),
                &f.node.public(),
                Capability::ExecuteInference,
                at(25),
            )
            .expect_err("an expired credential must not authorise");
        assert!(matches!(err, WalletError::Expired));
    }

    #[test]
    fn tampering_with_capabilities_invalidates_the_signature() {
        let mut f = fixture(vec![Capability::SyncState]);
        f.signed
            .credential
            .credential_subject
            .capabilities
            .push(Capability::ExecuteInference);

        let err = f
            .signed
            .verify_signature()
            .expect_err("escalating capabilities must break the signature");
        assert!(matches!(err, WalletError::InvalidSignature(_)));
    }

    #[test]
    fn tampering_with_the_validity_window_invalidates_the_signature() {
        let mut f = fixture(vec![Capability::ExecuteInference]);
        f.signed.credential.valid_until = Some(at(100_000));

        let err = f
            .signed
            .verify_signature()
            .expect_err("extending expiry must break the signature");
        assert!(matches!(err, WalletError::InvalidSignature(_)));
    }

    #[test]
    fn claiming_someone_elses_issuer_did_is_rejected() {
        // Self-signed but naming another owner as issuer.
        let attacker = SecretKey::generate();
        let victim = SecretKey::generate();
        let node = SecretKey::generate();

        let credential = Credential::delegation(
            did_key_from_endpoint(&victim.public()), // claims the victim issued it
            did_key_from_endpoint(&node.public()),
            vec![Capability::ExecuteInference],
            at(0),
            None,
        );
        let forged = sign_credential(credential, &attacker).expect("signing succeeds");

        let err = forged
            .verify_signature()
            .expect_err("the signer must be the stated issuer");
        assert!(matches!(err, WalletError::IssuerMismatch { .. }));
    }

    #[test]
    fn signed_credentials_round_trip_through_json() {
        // Credentials cross the wire; a round trip must preserve verifiability.
        let f = fixture(vec![Capability::ExecuteInference]);
        let json = serde_json::to_string(&f.signed).expect("serializes");
        let parsed: SignedCredential = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(parsed, f.signed);
        parsed
            .verify_signature()
            .expect("still verifies after transit");
    }
}
