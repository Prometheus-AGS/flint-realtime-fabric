// Native (non-wasm) test for the crdt_apply_delta WASM binding function.
// Tests the pure-Rust path — no browser required.

use loro::LoroDoc;

fn make_snapshot(key: &str, value: &str) -> Vec<u8> {
    let doc = LoroDoc::new();
    doc.get_map("data")
        .insert(key, value)
        .expect("insert should succeed");
    doc.export(loro::ExportMode::Snapshot)
        .expect("snapshot export should succeed")
}

fn make_delta(base: &[u8], after: &[u8]) -> Vec<u8> {
    let doc = LoroDoc::new();
    if !base.is_empty() {
        doc.import(base).expect("import base");
    }
    doc.import(after).expect("import after");
    doc.export(loro::ExportMode::all_updates())
        .expect("delta export")
}

fn read_str(snapshot: &[u8], key: &str) -> Option<String> {
    let doc = LoroDoc::new();
    doc.import(snapshot).ok()?;
    let map = doc.get_map("data");
    match map.get(key)?.into_value() {
        Ok(loro::LoroValue::String(s)) => Some(s.to_string()),
        _ => None,
    }
}

#[test]
fn crdt_apply_delta_roundtrips() {
    let snap_a = make_snapshot("author", "Prometheus");
    let snap_b = make_snapshot("project", "FRF");

    let delta_b = make_delta(&snap_a, &snap_b);

    let merged = frf_wasm::crdt_apply_delta(&snap_a, &delta_b);
    assert!(!merged.is_empty(), "merged snapshot should not be empty");

    let author = read_str(&merged, "author");
    let project = read_str(&merged, "project");

    assert_eq!(
        author.as_deref(),
        Some("Prometheus"),
        "author field survived merge"
    );
    assert_eq!(
        project.as_deref(),
        Some("FRF"),
        "project field present after merge"
    );
}

#[test]
fn crdt_apply_delta_empty_inputs_noop() {
    let snap = make_snapshot("x", "y");
    let result = frf_wasm::crdt_apply_delta(&snap, &[]);
    assert_eq!(result, snap, "empty delta should return original snapshot");

    let result2 = frf_wasm::crdt_apply_delta(&[], &snap);
    assert!(
        result2.is_empty(),
        "empty snapshot + any delta should return empty"
    );
}
