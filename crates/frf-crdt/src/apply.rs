use frf_ports::{ApplyDelta, PortError};

use crate::merge::apply_delta as loro_apply_delta;

/// Implements the `ApplyDelta` port using Loro as the merge engine.
///
/// Injected into `SyncUseCase` so `frf-app` never imports `frf-crdt`
/// directly — the dependency arrow remains adapter → ports → app.
#[derive(Debug, Clone, Default)]
pub struct LoroDeltaApplier;

impl ApplyDelta for LoroDeltaApplier {
    fn apply(&self, existing: &[u8], delta: &[u8]) -> Result<Vec<u8>, PortError> {
        loro_apply_delta(existing, delta).map_err(|e| PortError::Transport(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loro::LoroDoc;

    fn bytes_with(key: &str, val: &str) -> Vec<u8> {
        let doc = LoroDoc::new();
        doc.get_map("root").insert(key, val).unwrap();
        doc.export(loro::ExportMode::Snapshot).unwrap()
    }

    #[test]
    fn applier_merges_via_port_trait() {
        let applier = LoroDeltaApplier;
        let base = bytes_with("x", "hello");
        let delta = bytes_with("y", "world");

        let merged = applier.apply(&base, &delta).expect("apply");

        let doc = LoroDoc::new();
        doc.import(&merged).unwrap();
        let map = doc.get_map("root");
        assert!(map.get("x").is_some());
        assert!(map.get("y").is_some());
    }

    #[test]
    fn applier_handles_empty_base() {
        let applier = LoroDeltaApplier;
        let delta = bytes_with("k", "v");
        let merged = applier.apply(&[], &delta).expect("apply empty base");

        let doc = LoroDoc::new();
        doc.import(&merged).unwrap();
        assert!(doc.get_map("root").get("k").is_some());
    }
}
