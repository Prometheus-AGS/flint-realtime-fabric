use crate::error::CrdtFfiError;

/// Apply a Loro CRDT delta to an existing snapshot, returning a new snapshot.
///
/// Both `existing` and `delta` are engine-opaque bytes. Pass an empty `Vec`
/// for `existing` when no prior state exists.
///
/// Exposed to Swift, Kotlin, and Dart via `UniFFI`.
///
/// # Errors
///
/// Returns `CrdtFfiError::ApplyDelta` if the merge fails.
#[uniffi::export]
#[allow(clippy::needless_pass_by_value)]
pub fn crdt_apply_delta(existing: Vec<u8>, delta: Vec<u8>) -> Result<Vec<u8>, CrdtFfiError> {
    frf_crdt::apply_delta(&existing, &delta).map_err(|e| CrdtFfiError::ApplyDelta(e.to_string()))
}

/// Return an empty Loro snapshot (new document with no ops).
///
/// Useful for initialising a local entity store before the first server sync.
///
/// # Errors
///
/// Returns `CrdtFfiError::Encode` if the Loro document cannot be exported.
#[uniffi::export]
pub fn crdt_new_snapshot() -> Result<Vec<u8>, CrdtFfiError> {
    use loro::LoroDoc;
    LoroDoc::new()
        .export(loro::ExportMode::Snapshot)
        .map_err(|e| CrdtFfiError::Encode(e.to_string()))
}

/// Return the version number (Lamport clock) of a Loro snapshot.
///
/// Returns `0` if the bytes are empty or cannot be decoded.
#[must_use]
#[uniffi::export]
#[allow(clippy::needless_pass_by_value)]
pub fn crdt_snapshot_version(snapshot: Vec<u8>) -> u64 {
    if snapshot.is_empty() {
        return 0;
    }
    let doc = loro::LoroDoc::new();
    if doc.import(&snapshot).is_err() {
        return 0;
    }
    u64::try_from(doc.oplog_vv().values().copied().max().unwrap_or(0).max(0)).unwrap_or(0)
}
