use crate::error::DidError;

/// Translates a `did:web` identifier into the HTTPS URL of its DID document.
///
/// Per the `did:web` method: colons in the method-specific identifier become
/// path segments, a percent-encoded colon (`%3A`) denotes a port, and an
/// identifier with no path resolves to `/.well-known/did.json`.
///
/// This function performs no I/O — it only computes the URL, so the caller
/// chooses the HTTP client and its timeout, proxy, and TLS policy.
///
/// # Why `did:web` exists here alongside `did:key`
///
/// A `did:key` **cannot rotate**: the identifier is the key, so a compromised
/// node key means a permanently dead identity that must be re-paired
/// everywhere. `did:web` anchors the identifier to a domain instead, so the
/// document behind it can publish a new key while the identifier is unchanged.
/// The cost is that the identity becomes only as decentralized as the domain —
/// which is why it is an opt-in upgrade for nodes that control one, not the
/// default for a device in someone's home.
///
/// # Examples
///
/// ```
/// use frf_did::did_web_document_url;
///
/// assert_eq!(
///     did_web_document_url("did:web:example.com").unwrap(),
///     "https://example.com/.well-known/did.json"
/// );
/// assert_eq!(
///     did_web_document_url("did:web:example.com:nodes:studio").unwrap(),
///     "https://example.com/nodes/studio/did.json"
/// );
/// ```
///
/// # Errors
///
/// - [`DidError::UnsupportedMethod`] if `did` is not a `did:web` identifier.
/// - [`DidError::Malformed`] if the domain segment is empty.
pub fn did_web_document_url(did: &str) -> Result<String, DidError> {
    let ident = did
        .strip_prefix("did:web:")
        .ok_or_else(|| DidError::UnsupportedMethod(did.to_owned()))?;

    if ident.is_empty() {
        return Err(DidError::Malformed("did:web has no domain".to_owned()));
    }

    let mut segments = ident.split(':');
    let domain = segments
        .next()
        .filter(|d| !d.is_empty())
        .ok_or_else(|| DidError::Malformed("did:web has no domain".to_owned()))?
        // A percent-encoded colon carries the port.
        .replace("%3A", ":")
        .replace("%3a", ":");

    let path: Vec<&str> = segments.filter(|s| !s.is_empty()).collect();

    Ok(if path.is_empty() {
        format!("https://{domain}/.well-known/did.json")
    } else {
        format!("https://{domain}/{}/did.json", path.join("/"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_domain_uses_well_known() {
        assert_eq!(
            did_web_document_url("did:web:example.com").expect("valid did:web"),
            "https://example.com/.well-known/did.json"
        );
    }

    #[test]
    fn path_segments_replace_well_known() {
        assert_eq!(
            did_web_document_url("did:web:example.com:nodes:studio").expect("valid did:web"),
            "https://example.com/nodes/studio/did.json"
        );
    }

    #[test]
    fn percent_encoded_colon_becomes_a_port() {
        assert_eq!(
            did_web_document_url("did:web:localhost%3A8443").expect("valid did:web"),
            "https://localhost:8443/.well-known/did.json"
        );
    }

    #[test]
    fn always_resolves_over_https() {
        // did:web's trust derives entirely from TLS and DNS; a plaintext
        // resolution would silently void that.
        let url = did_web_document_url("did:web:example.com:a:b").expect("valid did:web");
        assert!(url.starts_with("https://"));
    }

    #[test]
    fn rejects_other_methods() {
        let err = did_web_document_url("did:key:z6Mkf5rGMoatrSj1f4CyvuHBeXJELe9RPdzo2PKGNCKVtZxP")
            .expect_err("did:key is not did:web");
        assert!(matches!(err, DidError::UnsupportedMethod(_)));
    }

    #[test]
    fn rejects_an_empty_domain() {
        let err = did_web_document_url("did:web:").expect_err("empty domain must be rejected");
        assert!(matches!(err, DidError::Malformed(_)));
    }
}
