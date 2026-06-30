use loro::LoroDoc;
use tracing::instrument;

use crate::error::CrdtError;

/// Merge `delta` into `existing`, returning a new snapshot.
///
/// Both `existing` and `delta` must be Loro-encoded bytes (snapshot or update
/// format). The result is always a full snapshot so downstream callers never
/// need to accumulate update history themselves.
///
/// # Errors
///
/// Returns `CrdtError` if `existing` or `delta` cannot be decoded, or if the
/// merged snapshot cannot be exported.
#[instrument(skip_all, fields(existing_len = existing.len(), delta_len = delta.len()))]
pub fn apply_delta(existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, CrdtError> {
    let doc = LoroDoc::new();

    if !existing.is_empty() {
        doc.import(existing)
            .map_err(|e| CrdtError::Decode(e.to_string()))?;
    }

    doc.import(delta)
        .map_err(|e| CrdtError::Merge(e.to_string()))?;

    doc.export(loro::ExportMode::Snapshot)
        .map_err(|e| CrdtError::Encode(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_snapshot(key: &str, value: &str) -> Vec<u8> {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert(key, value).expect("insert");
        doc.export(loro::ExportMode::Snapshot).expect("export")
    }

    fn make_update(key: &str, value: &str) -> Vec<u8> {
        let doc = LoroDoc::new();
        let map = doc.get_map("root");
        map.insert(key, value).expect("insert");
        doc.export(loro::ExportMode::all_updates())
            .expect("export update")
    }

    fn read_str(bytes: &[u8], key: &str) -> Option<String> {
        let doc = LoroDoc::new();
        doc.import(bytes).ok()?;
        let map = doc.get_map("root");
        let voc = map.get(key)?;
        match voc.into_value() {
            Ok(loro::LoroValue::String(s)) => Some(s.to_string()),
            _ => None,
        }
    }

    #[test]
    fn merges_disjoint_keys() {
        let base = make_snapshot("a", "1");
        let delta = make_update("b", "2");

        let merged = apply_delta(&base, &delta).expect("apply_delta");

        assert_eq!(read_str(&merged, "a").as_deref(), Some("1"));
        assert_eq!(read_str(&merged, "b").as_deref(), Some("2"));
    }

    #[test]
    fn apply_delta_to_empty_base() {
        let delta = make_snapshot("k", "v");
        let merged = apply_delta(&[], &delta).expect("apply_delta empty base");
        assert_eq!(read_str(&merged, "k").as_deref(), Some("v"));
    }

    #[test]
    fn convergence_commutative() {
        let doc_a = make_snapshot("x", "from_a");
        let doc_b = make_snapshot("y", "from_b");

        let a_then_b = apply_delta(&doc_a, &doc_b).expect("a then b");
        let b_then_a = apply_delta(&doc_b, &doc_a).expect("b then a");

        assert_eq!(read_str(&a_then_b, "x"), read_str(&b_then_a, "x"));
        assert_eq!(read_str(&a_then_b, "y"), read_str(&b_then_a, "y"));
    }
}
