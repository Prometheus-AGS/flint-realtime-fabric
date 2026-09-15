# Decisions — pri-c008-cdc-commit-mapping

- Use the pinned `pg_walstream 0.6.3` parser and compensate for its suppressed
  initial Relation event by seeding mappings from the validated live catalog.
- Treat an enrollment as a server-owned allowlist. Reject ambiguous publication,
  table, type, key, tenant and replica-identity mappings before readiness.
- Build stable identities from source epoch, commit LSN and transaction index.
  Keep the full hash in the payload and derive a deterministic UUID for the
  existing envelope interface.
- Publish a whole source commit before updating applied LSN. Stop on every poison
  or durable-publication error so later feedback cannot cross that commit.
- Re-read catalog metadata at each commit boundary to close the schema-change
  race created by parser-side Relation caching.
- Accept replay after an uncertain durable publish and rely on the c007 broker's
  stable-ID deduplication contract.
- Apply the 500-line maintainability rule to hand-authored source and config.
  Keep machine-generated dependency lockfiles in the single-file format their
  package manager requires and bind their exact hash in evidence.
