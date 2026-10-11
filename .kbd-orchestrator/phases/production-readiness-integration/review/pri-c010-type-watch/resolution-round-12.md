# c010 adversarial review round 12 resolution

## Critical: filtered events can advance after subscription revocation

Disposition: fixed. Every live or resumed delivery now rechecks the subscription
tuple after source-epoch validation and before checkpoint issuance, tenant/type
filtering, object authorization, or payload construction. Denial and
authorization errors terminate with `PermissionDenied`. The object check remains
separate and still converts object-denied events to opaque checkpoint progress
while subscription authority is valid.

A regression supplies a matching-epoch event from another tenant after the
subscription check begins denying and proves the result is a permission terminal,
not `CheckpointAdvanced`. Strict app Clippy and all seven stream tests pass.
