# c010 adversarial review round 6 resolution

## Critical: alleged missing `EntityRow::entity_type`

Disposition: rejected as a packet-context misread. The current
`crates/frf-projection-surreal/src/model.rs` declares
`EntityRow::entity_type: String`, and `EntityRow::from_change` populates it from
`change.entity_type`. The cited snapshot code therefore filters the stored row
before parsing its typed delivery, then independently checks the parsed mutation
with `EntityTypeSelector::matches`.

The affected crate passes an offline all-target compile and test run, the full
affected-crate strict Clippy matrix, and the real PostgreSQL-to-Surreal-to-tonic
run. No source change is required for this finding.
