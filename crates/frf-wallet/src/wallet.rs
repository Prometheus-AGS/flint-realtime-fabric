use chrono::{DateTime, Utc};
use iroh::PublicKey;

use crate::credential::Capability;
use crate::error::WalletError;
use crate::proof::SignedCredential;

/// Holds the credentials a node has been issued.
///
/// This is the holder role from the issuer/holder/verifier triad: the wallet
/// stores credentials and presents them, but never mints them. Issuance
/// requires the owner's secret key, which a node does not have.
#[derive(Debug, Clone, Default)]
pub struct Wallet {
    credentials: Vec<SignedCredential>,
}

impl Wallet {
    /// Creates an empty wallet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a credential after checking that its signature is valid.
    ///
    /// Rejecting unverifiable credentials at the door means a later
    /// presentation cannot fail for a reason the holder could have caught.
    ///
    /// # Errors
    ///
    /// Returns any error from [`SignedCredential::verify_signature`].
    pub fn store(&mut self, credential: SignedCredential) -> Result<(), WalletError> {
        credential.verify_signature()?;
        self.credentials.push(credential);
        Ok(())
    }

    /// Returns the number of held credentials.
    #[must_use]
    pub fn len(&self) -> usize {
        self.credentials.len()
    }

    /// Returns whether the wallet holds no credentials.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.credentials.is_empty()
    }

    /// Finds a held credential that authorises `capability` for `node_key` at
    /// `now`, issued by `owner_did`.
    ///
    /// Returns `None` when nothing in the wallet authorises the request — an
    /// empty wallet therefore authorises nothing, which is the posture a node
    /// starts in.
    #[must_use]
    pub fn present(
        &self,
        owner_did: &str,
        node_key: &PublicKey,
        capability: Capability,
        now: DateTime<Utc>,
    ) -> Option<&SignedCredential> {
        self.credentials.iter().find(|c| {
            c.verify_delegation(owner_did, node_key, capability, now)
                .is_ok()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credential::Credential;
    use crate::proof::sign_credential;
    use frf_did::did_key_from_endpoint;
    use iroh::SecretKey;

    fn at(offset_hours: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000 + offset_hours * 3600, 0)
            .unwrap_or(DateTime::UNIX_EPOCH)
    }

    fn delegation(
        owner: &SecretKey,
        node: &SecretKey,
        capabilities: Vec<Capability>,
    ) -> SignedCredential {
        let credential = Credential::delegation(
            did_key_from_endpoint(&owner.public()),
            did_key_from_endpoint(&node.public()),
            capabilities,
            at(0),
            Some(at(24)),
        );
        sign_credential(credential, owner).expect("signing succeeds")
    }

    #[test]
    fn an_empty_wallet_authorises_nothing() {
        let wallet = Wallet::new();
        let node = SecretKey::generate();
        assert!(wallet.is_empty());
        assert!(
            wallet
                .present(
                    "did:key:anyone",
                    &node.public(),
                    Capability::ExecuteInference,
                    at(1)
                )
                .is_none()
        );
    }

    #[test]
    fn a_stored_credential_can_be_presented() {
        let owner = SecretKey::generate();
        let node = SecretKey::generate();
        let mut wallet = Wallet::new();
        wallet
            .store(delegation(
                &owner,
                &node,
                vec![Capability::ExecuteInference],
            ))
            .expect("a valid credential is accepted");

        assert_eq!(wallet.len(), 1);
        assert!(
            wallet
                .present(
                    &did_key_from_endpoint(&owner.public()),
                    &node.public(),
                    Capability::ExecuteInference,
                    at(1),
                )
                .is_some()
        );
    }

    #[test]
    fn a_tampered_credential_is_refused_at_storage_time() {
        let owner = SecretKey::generate();
        let node = SecretKey::generate();
        let mut tampered = delegation(&owner, &node, vec![Capability::SyncState]);
        tampered
            .credential
            .credential_subject
            .capabilities
            .push(Capability::ExecuteInference);

        let mut wallet = Wallet::new();
        let err = wallet
            .store(tampered)
            .expect_err("a tampered credential must not be stored");
        assert!(matches!(err, WalletError::InvalidSignature(_)));
        assert!(wallet.is_empty(), "nothing is retained on failure");
    }

    #[test]
    fn presentation_selects_only_a_matching_capability() {
        let owner = SecretKey::generate();
        let node = SecretKey::generate();
        let owner_did = did_key_from_endpoint(&owner.public());

        let mut wallet = Wallet::new();
        wallet
            .store(delegation(&owner, &node, vec![Capability::SyncState]))
            .expect("stored");
        wallet
            .store(delegation(
                &owner,
                &node,
                vec![Capability::ExecuteInference],
            ))
            .expect("stored");

        let found = wallet
            .present(
                &owner_did,
                &node.public(),
                Capability::ExecuteInference,
                at(1),
            )
            .expect("a matching credential exists");
        assert!(found.credential.grants(Capability::ExecuteInference));

        assert!(
            wallet
                .present(
                    &owner_did,
                    &node.public(),
                    Capability::AcceptPeerSessions,
                    at(1)
                )
                .is_none(),
            "an undelegated capability must not be satisfiable"
        );
    }

    #[test]
    fn expired_credentials_are_not_presented() {
        let owner = SecretKey::generate();
        let node = SecretKey::generate();
        let mut wallet = Wallet::new();
        wallet
            .store(delegation(
                &owner,
                &node,
                vec![Capability::ExecuteInference],
            ))
            .expect("stored");

        assert!(
            wallet
                .present(
                    &did_key_from_endpoint(&owner.public()),
                    &node.public(),
                    Capability::ExecuteInference,
                    at(25),
                )
                .is_none(),
            "an expired credential must not be presentable"
        );
    }
}
