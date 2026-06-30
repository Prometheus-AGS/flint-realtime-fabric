use frf_domain::{EntityId, TenantId};

/// Composite key layout: `entity_uuid(16)` | `tenant_uuid(16)` | `local_seq(8 BE)`
/// = 40 bytes total.
///
/// The prefix ordering means a range scan from
/// `make_prefix(entity, tenant)` to `make_prefix_end(entity, tenant)`
/// covers all ops for that entity+tenant pair in ascending `local_seq` order.
pub const KEY_LEN: usize = 40;
pub const PREFIX_LEN: usize = 32;

pub fn make_key(entity_id: &EntityId, tenant_id: &TenantId, local_seq: u64) -> [u8; KEY_LEN] {
    let mut buf = [0u8; KEY_LEN];
    buf[..16].copy_from_slice(entity_id.as_uuid().as_bytes());
    buf[16..32].copy_from_slice(tenant_id.as_uuid().as_bytes());
    buf[32..40].copy_from_slice(&local_seq.to_be_bytes());
    buf
}

/// Lower bound (inclusive) for all ops of a given entity+tenant.
pub fn make_prefix(entity_id: &EntityId, tenant_id: &TenantId) -> [u8; PREFIX_LEN] {
    let mut buf = [0u8; PREFIX_LEN];
    buf[..16].copy_from_slice(entity_id.as_uuid().as_bytes());
    buf[16..32].copy_from_slice(tenant_id.as_uuid().as_bytes());
    buf
}

/// Upper bound (exclusive) for all ops of a given entity+tenant.
/// Increments the last byte of the prefix — safe because UUID bytes are
/// arbitrary; worst case wraps to the next tenant bucket.
pub fn make_prefix_end(entity_id: &EntityId, tenant_id: &TenantId) -> [u8; PREFIX_LEN] {
    let mut buf = make_prefix(entity_id, tenant_id);
    // Increment tenant bytes as a big-endian integer.
    let mut carry = true;
    for b in buf[16..32].iter_mut().rev() {
        if carry {
            let (nb, c) = b.overflowing_add(1);
            *b = nb;
            carry = c;
        } else {
            break;
        }
    }
    buf
}

/// Extract `local_seq` from the trailing 8 bytes of a composite key.
pub fn seq_from_key(key: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&key[32..40]);
    u64::from_be_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_roundtrip() {
        let eid = EntityId::new();
        let tid = TenantId::new();
        let k = make_key(&eid, &tid, 42);
        assert_eq!(seq_from_key(&k), 42);
    }

    #[test]
    fn prefix_end_greater_than_prefix() {
        let eid = EntityId::new();
        let tid = TenantId::new();
        let p = make_prefix(&eid, &tid);
        let e = make_prefix_end(&eid, &tid);
        assert!(e.as_slice() > p.as_slice());
    }
}
