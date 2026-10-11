# Refinement decisions — pri-c006-watch-contract

- Add `flint.v2.EntityService.WatchEntityType`; leave every current v1 byte
  unchanged and generate both Rust namespaces together.
- Keep source epoch/commit LSN/transaction index, stable event identity, broker
  partition/offset and opaque client checkpoint as separate concepts.
- Use a server-established snapshot/WAL barrier and resume strictly after the
  returned checkpoint; expired history and epoch/scope mismatches resnapshot.
- Map allowlisted PostgreSQL values to an explicit typed union and canonicalize
  primary keys by PK ordinal, including composite and non-UUID keys.
- Send key-only authorized deletes and opaque checkpoint-only progress over
  denied history; never expose old rows or denied mutation metadata.
- Use terminal typed controls for resnapshot and bounded-buffer lag. Transport
  cancellation stops further server production after observation.
- Record `frf_proto::fv2` as a pre-1.0 additive public API requiring at least a
  minor release. Defer domain/port API additions and runtime adapters to their
  owning changes while reserving one dedicated port per adapter.
