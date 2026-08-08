use iroh::EndpointId;

use crate::error::DidError;

/// Multicodec prefix identifying an Ed25519 public key (`ed25519-pub`).
///
/// `0xed` encoded as an unsigned varint is the two bytes `0xed 0x01`. This is
/// why every Ed25519 `did:key` renders with the familiar `z6Mk` prefix — the
/// constant leading bytes dominate the base58 encoding.
const ED25519_PUB_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// The `did:key` method prefix, including the multibase `z` tag for base58btc.
const DID_KEY_PREFIX: &str = "did:key:z";

/// Length of a raw Ed25519 public key.
const ED25519_KEY_LEN: usize = 32;

/// Derives the `did:key` identifier for an iroh endpoint.
///
/// This is the bridge that makes decentralized identity nearly free here: an
/// iroh `EndpointId` **is** an Ed25519 public key (`iroh_base::EndpointId` is a
/// type alias for `PublicKey`), and a `did:key` is that same key under a
/// multicodec prefix and base58btc multibase encoding. No new key material, no
/// registry, and no network access — the derivation is a pure function.
///
/// # Examples
///
/// ```
/// use frf_did::did_key_from_bytes;
///
/// // The Ed25519 example from the W3C did:key specification.
/// let key = [
///     0x09, 0x5f, 0x9a, 0x1a, 0x59, 0x5d, 0xde, 0x75, 0x5d, 0x82, 0x78, 0x68,
///     0x64, 0xad, 0x03, 0xdf, 0xa5, 0xa4, 0xfb, 0xd6, 0x88, 0x32, 0x56, 0x63,
///     0x64, 0xe2, 0xb6, 0x5e, 0x13, 0xcc, 0x9e, 0x44,
/// ];
/// assert_eq!(
///     did_key_from_bytes(&key),
///     "did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP"
/// );
/// ```
#[must_use]
pub fn did_key_from_endpoint(endpoint_id: &EndpointId) -> String {
    did_key_from_bytes(endpoint_id.as_bytes())
}

/// Derives a `did:key` identifier from raw Ed25519 public key bytes.
///
/// See [`did_key_from_endpoint`] for the rationale.
#[must_use]
pub fn did_key_from_bytes(key: &[u8; ED25519_KEY_LEN]) -> String {
    let mut payload = Vec::with_capacity(ED25519_PUB_MULTICODEC.len() + ED25519_KEY_LEN);
    payload.extend_from_slice(&ED25519_PUB_MULTICODEC);
    payload.extend_from_slice(key);

    let mut did = String::from(DID_KEY_PREFIX);
    did.push_str(&bs58::encode(&payload).into_string());
    did
}

/// Recovers the Ed25519 public key from a `did:key` identifier.
///
/// This is the inverse of [`did_key_from_bytes`]. It is what a peer uses to
/// check that a DID presented in a credential actually corresponds to the key
/// that authenticated the QUIC session — without it, a peer could present
/// someone else's DID alongside its own proven key.
///
/// # Errors
///
/// - [`DidError::UnsupportedMethod`] if the identifier is not `did:key:z…`.
/// - [`DidError::Malformed`] if the base58 payload cannot be decoded.
/// - [`DidError::UnsupportedKeyType`] if the multicodec prefix is not Ed25519.
/// - [`DidError::Malformed`] if the key is not exactly 32 bytes.
pub fn public_key_from_did_key(did: &str) -> Result<[u8; ED25519_KEY_LEN], DidError> {
    let encoded = did
        .strip_prefix(DID_KEY_PREFIX)
        .ok_or_else(|| DidError::UnsupportedMethod(did.to_owned()))?;

    let payload = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| DidError::Malformed(format!("base58 decode failed: {e}")))?;

    let (prefix, key) = payload
        .split_at_checked(ED25519_PUB_MULTICODEC.len())
        .ok_or_else(|| DidError::Malformed("payload shorter than multicodec prefix".to_owned()))?;

    if prefix != ED25519_PUB_MULTICODEC {
        return Err(DidError::UnsupportedKeyType(format!(
            "expected ed25519-pub (ed01), found {}",
            hex_prefix(prefix)
        )));
    }

    key.try_into().map_err(|_| {
        DidError::Malformed(format!(
            "expected a {ED25519_KEY_LEN}-byte key, found {}",
            key.len()
        ))
    })
}

/// Returns whether `did` denotes exactly the key behind `endpoint_id`.
///
/// Peers should call this before trusting any DID a remote asserts: the QUIC
/// handshake proves the *key*, so a DID is only meaningful once it is shown to
/// be that same key. A DID that does not match is not merely unverified — it is
/// an impersonation attempt.
#[must_use]
pub fn did_matches_endpoint(did: &str, endpoint_id: &EndpointId) -> bool {
    match public_key_from_did_key(did) {
        Ok(key) => &key == endpoint_id.as_bytes(),
        Err(_) => false,
    }
}

fn hex_prefix(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    bytes.iter().fold(String::new(), |mut acc, b| {
        // Writing into a String is infallible, so the result is discarded
        // rather than unwrapped (the workspace denies unwrap/expect).
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Published Ed25519 example from the W3C `did:key` specification.
    ///
    /// The key bytes were recovered by decoding the published identifier, and
    /// re-encoding them reproduces that identifier exactly — so this pair is an
    /// external check, not a value this implementation produced.
    const SPEC_DID: &str = "did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP";
    const SPEC_KEY: [u8; 32] = [
        0x09, 0x5f, 0x9a, 0x1a, 0x59, 0x5d, 0xde, 0x75, 0x5d, 0x82, 0x78, 0x68, 0x64, 0xad, 0x03,
        0xdf, 0xa5, 0xa4, 0xfb, 0xd6, 0x88, 0x32, 0x56, 0x63, 0x64, 0xe2, 0xb6, 0x5e, 0x13, 0xcc,
        0x9e, 0x44,
    ];

    #[test]
    fn matches_the_w3c_specification_test_vector() {
        assert_eq!(did_key_from_bytes(&SPEC_KEY), SPEC_DID);
    }

    #[test]
    fn decodes_the_w3c_specification_test_vector() {
        let key = public_key_from_did_key(SPEC_DID).expect("spec vector must decode");
        assert_eq!(key, SPEC_KEY);
    }

    #[test]
    fn ed25519_dids_carry_the_z6mk_prefix() {
        // A property of the ed01 multicodec prefix, and a cheap guard against
        // silently emitting a different key type.
        assert!(did_key_from_bytes(&SPEC_KEY).starts_with("did:key:z6Mk"));
    }

    #[test]
    fn derivation_round_trips_for_an_iroh_key() {
        let key = iroh::SecretKey::generate();
        let endpoint_id = key.public();

        let did = did_key_from_endpoint(&endpoint_id);
        let recovered = public_key_from_did_key(&did).expect("round trip must decode");

        assert_eq!(&recovered, endpoint_id.as_bytes());
        assert!(did_matches_endpoint(&did, &endpoint_id));
    }

    #[test]
    fn derivation_is_deterministic_and_offline() {
        // Two calls on the same key must agree; nothing here touches the network.
        let endpoint_id = iroh::SecretKey::generate().public();
        assert_eq!(
            did_key_from_endpoint(&endpoint_id),
            did_key_from_endpoint(&endpoint_id)
        );
    }

    #[test]
    fn distinct_keys_produce_distinct_dids() {
        let a = iroh::SecretKey::generate().public();
        let b = iroh::SecretKey::generate().public();
        assert_ne!(did_key_from_endpoint(&a), did_key_from_endpoint(&b));
    }

    #[test]
    fn rejects_a_did_belonging_to_another_key() {
        // The impersonation case: a valid DID that is not the peer's own key.
        let mine = iroh::SecretKey::generate().public();
        let theirs = iroh::SecretKey::generate().public();
        let their_did = did_key_from_endpoint(&theirs);

        assert!(!did_matches_endpoint(&their_did, &mine));
    }

    #[test]
    fn rejects_non_did_key_methods() {
        let err = public_key_from_did_key("did:web:example.com")
            .expect_err("did:web is not decodable as did:key");
        assert!(matches!(err, DidError::UnsupportedMethod(_)));
    }

    #[test]
    fn rejects_a_non_ed25519_multicodec() {
        // 0xe7 0x01 is secp256k1-pub, a valid did:key type we do not support here.
        let mut payload = vec![0xe7, 0x01];
        payload.extend_from_slice(&SPEC_KEY);
        let did = format!("did:key:z{}", bs58::encode(&payload).into_string());

        let err = public_key_from_did_key(&did).expect_err("secp256k1 must be rejected");
        assert!(matches!(err, DidError::UnsupportedKeyType(_)));
    }

    #[test]
    fn rejects_a_truncated_key() {
        let mut payload = ED25519_PUB_MULTICODEC.to_vec();
        payload.extend_from_slice(&SPEC_KEY[..16]);
        let did = format!("did:key:z{}", bs58::encode(&payload).into_string());

        let err = public_key_from_did_key(&did).expect_err("a 16-byte key must be rejected");
        assert!(matches!(err, DidError::Malformed(_)));
    }

    #[test]
    fn rejects_invalid_base58() {
        // '0' is not in the base58btc alphabet.
        let err = public_key_from_did_key("did:key:z000invalid000")
            .expect_err("invalid base58 must be rejected");
        assert!(matches!(err, DidError::Malformed(_)));
    }
}
