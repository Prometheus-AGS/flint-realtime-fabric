use std::time::Duration;

use jsonwebtoken::jwk::JwkSet;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;
use tracing::debug;

use crate::error::IdentityError;

/// Default refresh age stays inside the accepted five-second authority lifetime.
pub const DEFAULT_JWKS_MAX_AGE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone)]
struct CachedJwks {
    keys: JwkSet,
    fetched_at: Instant,
}

/// Age-bounded JWKS cache. Removed signing keys cannot remain trusted indefinitely.
#[derive(Debug)]
pub struct JwksCache {
    state: RwLock<Option<CachedJwks>>,
    refresh: Mutex<()>,
    max_age: Duration,
}

/// Create a new empty JWKS cache.
#[must_use]
pub fn new_cache() -> JwksCache {
    new_cache_with_max_age(DEFAULT_JWKS_MAX_AGE)
}

/// Create an empty cache with an explicit maximum age.
#[must_use]
pub fn new_cache_with_max_age(max_age: Duration) -> JwksCache {
    JwksCache {
        state: RwLock::new(None),
        refresh: Mutex::new(()),
        max_age,
    }
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
    // Serialize fetches so an older in-flight response cannot overwrite a
    // newer key set after rotation.
    let _refresh = cache.refresh.lock().await;
    debug!(url, "fetching JWKS");
    let jwks: JwkSet = http
        .get(url)
        .send()
        .await
        .map_err(|e| IdentityError::JwksFetch(e.to_string()))?
        .json()
        .await
        .map_err(|e| IdentityError::JwksFetch(e.to_string()))?;

    let mut guard = cache.state.write().await;
    *guard = Some(CachedJwks {
        keys: jwks.clone(),
        fetched_at: Instant::now(),
    });
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
        let guard = cache.state.read().await;
        if let Some(cached) = guard.as_ref()
            && cached.fetched_at.elapsed() < cache.max_age
        {
            return Ok(cached.keys.clone());
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
