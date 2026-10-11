# c010 semver and compatibility decision

- `proto/flint/v1/**` remains byte-identical to the c006 baseline. No v1 field,
  service, package, or generated namespace changes.
- The already-frozen `flint.v2.EntityService.WatchEntityType` now gains a server
  implementation and its generated tonic client is compiled by a Forge-facing
  surface test. This is additive wire behavior.
- `frf-domain` adds public canonical mutation/watch types and `frf-ports` adds
  `EntityTypeWatchSource` plus additive typed-projection methods. These are
  pre-1.0 public API additions and require at least the coordinated minor release
  already assigned to c023. The workspace version is not advanced mid-phase.
- Existing `EntityStore` implementors remain source-compatible because the new
  typed methods have fail-closed defaults. Production SurrealDB and gateway
  memory adapters override both methods.
- Rollback removes the v2 service route. Typed projection JSON remains optional
  and readable by the prior row model; a later forward rollout replays or
  resnapshots rows that lack typed metadata before serving v2 snapshots.
