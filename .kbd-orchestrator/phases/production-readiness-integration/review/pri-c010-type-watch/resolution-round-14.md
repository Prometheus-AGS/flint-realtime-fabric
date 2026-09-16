# c010 adversarial review round 14 resolution

## Critical: adapter constructors expose `anyhow`

Disposition: fixed. `frf-broker-iggy` no longer depends on `anyhow`.
`IggyBroker::new` and `with_replay_retention` return the adapter's
`IggyBrokerError`; invalid retention uses a new configuration variant and Iggy
connection failures use its transport variant. Gateway composition performs the
binary-edge conversion.

## Critical: snapshot barrier discloses filtered source progress

Disposition: fixed. Snapshot filtering and object authorization now complete
before the public barrier is selected. `WatchAccepted.snapshot_barrier`,
`SnapshotComplete.barrier`, and snapshot IDs use the greatest source position
among authorized in-scope rows, or the neutral configured-epoch position when
there are none. The durable global projection offset remains only inside the
encrypted checkpoint and still drives live replay after the atomic snapshot.

The isolation fixture includes later other-type, other-tenant, and object-denied
rows; its regression proves the one visible authorized row and barrier LSN 100,
instead of the global cursor's later LSN 103. Strict affected-crate Clippy, all
11 broker tests, and all three app integration tests pass.
