# c009 semver impact

The v1 protobuf contract is unchanged: no files under `proto/` or
`crates/frf-proto/` changed.

The Rust `EntityStore` port adds three required methods and new public projection
types. This is a breaking source change for external implementations of that
unstable internal Rust trait and would require a major version if `frf-ports`
were published with a stable semver promise. Every in-repository implementation
was updated atomically.

`LogBroker::head_offset` is additive and has a fail-closed default. The Iggy
adapter overrides it so projection readiness can prove catch-up; other broker
implementations remain source-compatible and cannot accidentally claim that
capability.

Gateway deployment configuration adds required `ENTITY_PROJECTION_*` values when
`CDC_ENABLED=true`, plus a pinned SurrealDB service and durable volume in the
reference full profile. The reference profile intentionally provisions SurrealDB
and requires its `FRF_SURREAL_*` renderer inputs even while CDC is disabled, so
adopters of that profile have a deployment-configuration migration. A custom
CDC-disabled gateway deployment retains the existing in-memory read behavior and
does not require `ENTITY_PROJECTION_*` values at gateway validation time.
