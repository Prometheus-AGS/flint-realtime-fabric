use std::sync::Arc;

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::RwLock;
use tracing::debug;

use crate::error::IdentityError;

pub type JwksCache = Arc<RwLock<Option<JwkSet>>>;

/// Create a new empty JWKS cache.
#[must_use]
pub fn new_cache() -> JwksCache {
    Arc::new(RwLock::new(None))
}

/// Fetch the JWKS from `url`, store it in `cache`, and return it.
///
/// # Errors
///
/// Returns [`IdentityError::JwksFetch`] if the HTTP request or JSON parsing fails.
pub async fn fetch_and_cache(
    http: &reqwest::Client,
    url: &str,
    cache: &JwksCache,
) -> Result<JwkSet, IdentityError> {
    debug!(url, "fetching JWKS");
    let jwks: JwkSet = http
        .get(url)
        .send()
        .await
        .map_err(|e| IdentityError::JwksFetch(e.to_string()))?
        .json()
        .await
        .map_err(|e| IdentityError::JwksFetch(e.to_string()))?;

    let mut guard = cache.write().await;
    *guard = Some(jwks.clone());
    Ok(jwks)
}

/// Return the cached JWKS, fetching it on first access.
///
/// # Errors
///
/// Returns [`IdentityError::JwksFetch`] if the HTTP request or JSON parsing fails.
pub async fn get_or_fetch(
    http: &reqwest::Client,
    url: &str,
    cache: &JwksCache,
) -> Result<JwkSet, IdentityError> {
    {
        let guard = cache.read().await;
        if let Some(ref jwks) = *guard {
            return Ok(jwks.clone());
        }
    }
    fetch_and_cache(http, url, cache).await
}

/// Force a JWKS re-fetch, replacing the cache (key rotation).
///
/// # Errors
///
/// Returns [`IdentityError::JwksFetch`] if the HTTP request or JSON parsing fails.
pub async fn refresh(
    http: &reqwest::Client,
    url: &str,
    cache: &JwksCache,
) -> Result<JwkSet, IdentityError> {
    debug!(url, "refreshing JWKS cache (key rotation)");
    fetch_and_cache(http, url, cache).await
}
