use frf_domain::TenantId;
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

/// Returns the single partition used by the channel adapter.
///
/// Topics are created with one partition. Until the `LogBroker` port carries
/// partition metadata, routing a consumer to a hash-derived partition can
/// select a partition that does not exist and cannot preserve global channel
/// order.
#[must_use]
pub const fn partition_id(_consumer_id: &str) -> u32 {
    1
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
        assert_eq!(a, 1, "single-partition topics must use partition 1");
    }
}
