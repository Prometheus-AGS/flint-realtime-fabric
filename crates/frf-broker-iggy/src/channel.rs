//! Iggy naming for the channel adapter.
//!
//! Streams are named `channel-{channel_id}` and the topic is the constant `"events"`
//! (see `broker.rs`). Neither derives from the tenant id or the channel path.
//!
//! This module previously exported `stream_name(TenantId) -> "tenant-{id}"` and
//! `topic_name(&str)`. Commit `26e4dfc` switched the adapter to `channel-{id}` naming
//! and dropped their only callers, but left the functions and their tests in place —
//! so a green suite kept asserting a `tenant-` prefix that no code produced. They are
//! removed rather than left as a misleading description of the wire format.

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

    #[test]
    fn consumer_partition_is_stable() {
        let a = partition_id("consumer-abc");
        let b = partition_id("consumer-abc");
        assert_eq!(a, b, "partition_id must be deterministic");
        assert_eq!(a, 1, "single-partition topics must use partition 1");
    }
}
