/// Phase-3 exit criterion: offline CRDT convergence roundtrip.
///
/// Simulates two devices diverging offline, then merging via `apply_delta`.
/// Asserts semantic convergence: both merged documents resolve to the same
/// logical value for all keys. (Loro snapshots embed internal metadata that
/// differs per-export, so byte identity is not the correct invariant here.)
use frf_crdt::apply_delta;
use loro::{LoroDoc, LoroValue};

fn device_snapshot(key: &str, value: &str) -> Vec<u8> {
    let doc = LoroDoc::new();
    let map = doc.get_map("entity");
    map.insert(key, value).expect("insert should succeed");
    doc.export(loro::ExportMode::Snapshot)
        .expect("snapshot export should succeed")
}

fn delta_since(before: &[u8], after_snapshot: &[u8]) -> Vec<u8> {
    let doc = LoroDoc::new();
    if !before.is_empty() {
        doc.import(before).expect("import before-state");
    }
    doc.import(after_snapshot).expect("import after-state");
    doc.export(loro::ExportMode::all_updates())
        .expect("delta export")
}

fn read_str(snapshot: &[u8], key: &str) -> Option<String> {
    let doc = LoroDoc::new();
    doc.import(snapshot).ok()?;
    let map = doc.get_map("entity");
    match map.get(key)?.into_value() {
        Ok(LoroValue::String(s)) => Some(s.to_string()),
        _ => None,
    }
}

#[test]
fn offline_roundtrip_converges() {
    // Device A: sets name = "Alice"
    let snap_a = device_snapshot("name", "Alice");
    // Device B: sets name = "Bob" (independent doc, no shared history yet)
    let snap_b = device_snapshot("name", "Bob");

    let delta_a = delta_since(&[], &snap_a);
    let delta_b = delta_since(&[], &snap_b);

    // Merge: apply B's delta onto A's snapshot
    let merged_from_a =
        apply_delta(&snap_a, &delta_b).expect("apply B delta onto A should succeed");

    // Merge: apply A's delta onto B's snapshot
    let merged_from_b =
        apply_delta(&snap_b, &delta_a).expect("apply A delta onto B should succeed");

    // Both merged docs must resolve "name" to the same value (Loro LWW winner)
    let val_a = read_str(&merged_from_a, "name");
    let val_b = read_str(&merged_from_b, "name");

    assert!(val_a.is_some(), "merged_from_a must contain 'name'");
    assert!(val_b.is_some(), "merged_from_b must contain 'name'");
    assert_eq!(
        val_a, val_b,
        "CRDT merge must be semantically convergent — both sides resolve to the same winner"
    );
}

#[test]
fn apply_delta_to_empty_base_is_idempotent() {
    let snap = device_snapshot("author", "Prometheus");
    let delta = delta_since(&[], &snap);

    // Applying a delta to an empty doc should produce a valid merged doc
    let merged = apply_delta(&[], &delta).expect("apply delta to empty base");

    // Re-applying the same delta must be idempotent (Loro deduplicates ops)
    let merged2 = apply_delta(&merged, &delta).expect("re-apply same delta");

    let v1 = read_str(&merged, "author");
    let v2 = read_str(&merged2, "author");
    assert_eq!(
        v1.as_deref(),
        Some("Prometheus"),
        "merged doc must contain author=Prometheus"
    );
    assert_eq!(v1, v2, "re-applying same delta must be idempotent semantically");
}

#[test]
fn three_way_merge_converges() {
    let snap_a = device_snapshot("field_a", "value_a");
    let snap_b = device_snapshot("field_b", "value_b");
    let snap_c = device_snapshot("field_c", "value_c");

    let delta_a = delta_since(&[], &snap_a);
    let delta_b = delta_since(&[], &snap_b);
    let delta_c = delta_since(&[], &snap_c);

    // Order 1: A ← B ← C
    let order1 = {
        let ab = apply_delta(&snap_a, &delta_b).unwrap();
        apply_delta(&ab, &delta_c).unwrap()
    };

    // Order 2: C ← A ← B
    let order2 = {
        let ca = apply_delta(&snap_c, &delta_a).unwrap();
        apply_delta(&ca, &delta_b).unwrap()
    };

    // Order 3: B ← C ← A
    let order3 = {
        let bc = apply_delta(&snap_b, &delta_c).unwrap();
        apply_delta(&bc, &delta_a).unwrap()
    };

    // All three orderings must agree on every key
    for key in ["field_a", "field_b", "field_c"] {
        let v1 = read_str(&order1, key);
        let v2 = read_str(&order2, key);
        let v3 = read_str(&order3, key);
        assert!(v1.is_some(), "order1 must contain '{key}'");
        assert_eq!(
            v1, v2,
            "three-way merge: '{key}' must be same in order1 and order2"
        );
        assert_eq!(
            v2, v3,
            "three-way merge: '{key}' must be same in order2 and order3"
        );
    }
}
