# c010 adversarial review round 5 resolution

## Critical: snapshot rows trust projection scope

Disposition: fixed. Snapshot delivery now independently verifies both tenant
identity and the requested entity-type selector before object authorization or
payload construction. Out-of-scope rows are skipped without exposing their keys
or records. The application integration fixture deliberately returns rows from
other tenants and types; the existing cross-scope test proves both subscribers
receive only the requested, object-authorized snapshot row.

## Warning: public enums are exhaustively matchable

Disposition: fixed. `TypeWatchStart`, `ResnapshotReason`, and
`EntityTypeWatchFrame` are now `#[non_exhaustive]`. The gateway handles future
frame variants as `DATA_LOSS` and maps future resnapshot reasons to the protocol's
unsupported-checkpoint recovery reason until a matching wire variant exists.
