# c010 adversarial review round 8 resolution

## Critical: snapshot Accepted checkpoint skips undelivered rows

Disposition: fixed. Snapshot admission now issues the `Accepted` checkpoint at
the pre-snapshot beginning position. The barrier checkpoint remains exclusively
in `SnapshotComplete`, after every authorized snapshot row. The deterministic
race test now disconnects after retaining the `Accepted` checkpoint, resumes,
and proves broker offset zero is replayed; the previous barrier checkpoint would
have started at offset one and failed this regression.

## Critical: stale acceptance test-log hash

Disposition: fixed. `acceptance-a.md` now records the capture time and test-log
SHA-256 from the current tracked `runtime-receipt.json` source-bound run.
