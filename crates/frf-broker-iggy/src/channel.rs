use frf_domain::TenantId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Maps a `TenantId` to an Iggy stream name (one stream per tenant).
#[must_use]
pub fn stream_name(tenant_id: TenantId) -> String {
    format!("tenant-{tenant_id}")
}

/// Maps a channel path to an Iggy topic name.
///
/// Iggy topic names must not contain `/` — this replaces all `/` with `_`.
#[must_use]
pub fn topic_name(path: &str) -> String {
    path.replace('/', "_")
}

/// Derives a consistent partition ID from a consumer ID string.
///
/// Returns a value in `[1, 8]`.
#[must_use]
pub fn partition_id(consumer_id: &str) -> u32 {
    let mut hasher = DefaultHasher::new();
    consumer_id.hash(&mut hasher);
    // value in [0,7], so +1 gives [1,8]; fits in u32 because max is 8
    #[allow(clippy::cast_possible_truncation)]
    let id = (hasher.finish() % 8 + 1) as u32;
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn tenant_maps_to_stream() {
        let id = TenantId::from_uuid(Uuid::nil());
        let name = stream_name(id);
        assert!(
            name.starts_with("tenant-"),
            "expected 'tenant-' prefix, got {name}"
        );
        assert!(!name.is_empty());
    }

    #[test]
    fn path_encodes_without_slash() {
        let topic = topic_name("entity/user/updates");
        assert!(
            !topic.contains('/'),
            "topic name must not contain '/', got {topic}"
        );
        assert_eq!(topic, "entity_user_updates");
    }

    #[test]
    fn empty_path_encodes_without_panic() {
        let topic = topic_name("");
        assert_eq!(topic, "");
    }

    #[test]
    fn consumer_partition_is_stable() {
        let a = partition_id("consumer-abc");
        let b = partition_id("consumer-abc");
        assert_eq!(a, b, "partition_id must be deterministic");
        assert!((1..=8).contains(&a), "partition {a} must be in [1,8]");
    }
}
