use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Composite key: `(subject, relation, object)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub String, pub String, pub String);

/// Cached check result with TTL.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub allowed: bool,
    pub expires_at: Instant,
}

/// Thread-safe TTL cache for `AuthzProvider::check` results.
#[derive(Debug, Default)]
pub struct CheckCache {
    entries: DashMap<CacheKey, CacheEntry>,
}

impl CheckCache {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Return cached result if present and unexpired.
    #[must_use]
    pub fn get(&self, key: &CacheKey) -> Option<bool> {
        let entry = self.entries.get(key)?;
        if entry.expires_at > Instant::now() {
            Some(entry.allowed)
        } else {
            drop(entry);
            self.entries.remove(key);
            None
        }
    }

    /// Insert a result with a TTL.
    pub fn insert(&self, key: CacheKey, allowed: bool, ttl_secs: u64) {
        self.entries.insert(
            key,
            CacheEntry {
                allowed,
                expires_at: Instant::now() + Duration::from_secs(ttl_secs),
            },
        );
    }

    /// Remove all entries where `(relation, object)` match the given values.
    pub fn invalidate_object(&self, relation: &str, object: &str) {
        self.entries.retain(|k, _| k.1 != relation || k.2 != object);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_returns_value() {
        let cache = CheckCache::new();
        let key = CacheKey("user".to_owned(), "view".to_owned(), "doc-1".to_owned());
        cache.insert(key.clone(), true, 60);
        assert_eq!(cache.get(&key), Some(true));
    }

    #[test]
    fn expired_entry_returns_none() {
        let cache = CheckCache::new();
        let key = CacheKey("user".to_owned(), "view".to_owned(), "doc-2".to_owned());
        // TTL 0 → immediately expired
        cache.entries.insert(
            key.clone(),
            CacheEntry {
                allowed: true,
                expires_at: Instant::now().checked_sub(Duration::from_secs(1)).unwrap(),
            },
        );
        assert_eq!(cache.get(&key), None);
    }

    #[test]
    fn invalidate_removes_matching() {
        let cache = CheckCache::new();
        let k1 = CacheKey("u1".to_owned(), "view".to_owned(), "doc".to_owned());
        let k2 = CacheKey("u2".to_owned(), "view".to_owned(), "doc".to_owned());
        let k3 = CacheKey("u1".to_owned(), "edit".to_owned(), "doc".to_owned());
        cache.insert(k1.clone(), true, 60);
        cache.insert(k2.clone(), true, 60);
        cache.insert(k3.clone(), true, 60);

        cache.invalidate_object("view", "doc");

        assert_eq!(cache.get(&k1), None, "k1 should be invalidated");
        assert_eq!(cache.get(&k2), None, "k2 should be invalidated");
        assert_eq!(
            cache.get(&k3),
            Some(true),
            "k3 different relation, should survive"
        );
    }
}
