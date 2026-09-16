# c010 adversarial review round 9 resolution

## Critical: alleged `Ok(false)` outer return mismatch

Disposition: clarified in source. The finding repeated round 4 and referred to
the nested transaction-preparation result, which was explicitly typed as
`Result<bool, PortError>` and mapped `false` to
`ProjectionApply::Duplicate` by the outer method. To remove the ambiguous packet
shape, preparation now returns the named `ProjectionPreparation::{Ready,
Duplicate}` enum. The outer adapter still cancels the duplicate transaction and
returns `ProjectionApply::Duplicate`.

Strict all-target Clippy and the adapter's all-target local test pass after this
clarity-only refactor.
