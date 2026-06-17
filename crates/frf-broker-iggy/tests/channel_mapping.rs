use frf_broker_iggy::channel::{partition_id, stream_name, topic_name};
use frf_domain::TenantId;
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
fn consumer_partition_is_stable() {
    let a = partition_id("consumer-abc");
    let b = partition_id("consumer-abc");
    assert_eq!(a, b, "partition_id must be deterministic");
    assert!(a >= 1, "partition must be >= 1");
    assert!(a <= 8, "partition must be <= 8");
}

#[test]
fn different_consumers_may_differ() {
    let a = partition_id("consumer-abc");
    let b = partition_id("consumer-xyz");
    // Not guaranteed to differ, but both must be in range
    assert!((1..=8).contains(&a), "a={a} out of range");
    assert!((1..=8).contains(&b), "b={b} out of range");
}
