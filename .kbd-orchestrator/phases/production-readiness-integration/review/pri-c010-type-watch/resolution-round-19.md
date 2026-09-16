# c010 adversarial review round 19 resolution

## Critical: token expiry can occur during authorization and enqueue

Disposition: fixed. The producer deadline now races periodic identity and
subscription checks, live per-delivery authorization/frame construction, every
initial-frame authorization check, and every bounded initial enqueue. Live
delivery also checks the deadline immediately before the non-awaiting
`try_send`. Expiry emits a permission terminal that preempts buffered protected
frames.

Two regressions hold authorization past a ten-millisecond token deadline. One
covers a live mutation and the other an initial snapshot frame; both prove the
output channel remains empty and the only terminal reports expiry. All nine
stream ordering, security, backpressure, and recovery tests plus strict pedantic
Clippy pass.
