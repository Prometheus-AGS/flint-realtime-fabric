use wasm_bindgen::prelude::*;

/// Apply a Loro CRDT delta to a snapshot and return the merged snapshot bytes.
///
/// Both `snapshot` and `delta` are opaque byte arrays produced by
/// `frf_crdt::apply_delta` / `frf_crdt::export_snapshot`. This function is
/// a thin WASM-friendly wrapper — the merge logic lives in `frf-crdt`.
///
/// Returns the original snapshot bytes if either input is empty (no-op).
/// Returns an empty `Vec<u8>` on merge error (callers should check length > 0).
#[must_use]
#[wasm_bindgen]
pub fn crdt_apply_delta(snapshot: &[u8], delta: &[u8]) -> Vec<u8> {
    if snapshot.is_empty() || delta.is_empty() {
        return snapshot.to_vec();
    }
    frf_crdt::apply_delta(snapshot, delta).unwrap_or_default()
}
