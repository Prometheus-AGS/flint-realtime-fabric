# c010 adversarial review round 11 resolution

## Critical: initial snapshot frames outlive revoked authority

Disposition: fixed. Before each initial frame enqueue, the producer revalidates
the bearer identity and subscription tuple inside the configured authority
interval. Denial, verifier error, authorization error, or timeout terminates via
`PermissionDenied` before that frame enters the client queue. A sequenced-auth
regression allows the first initial frame, revokes authority, and proves the
second frame is absent.

## Critical: resnapshot terminal can repeat

Disposition: fixed. Output state now terminates after any `Terminal::Frame`, not
only the `Lagged` variant. A regression forces the asynchronous terminal-change
branch, observes one `ResnapshotRequired`, and proves the following poll ends
the stream.

The stream tests moved into a 253-line sibling module to preserve the file-size
gate. Strict app Clippy and all six focused tests pass.
