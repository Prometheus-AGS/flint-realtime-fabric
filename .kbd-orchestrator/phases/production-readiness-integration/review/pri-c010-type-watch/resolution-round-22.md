# c010 adversarial review round 22 resolution

## Critical: revocation during object authorization could precede enqueue

Disposition: fixed. After live frame construction finishes—including any
awaited object authorization—the producer now re-verifies identity and
subscription authority before its non-awaiting enqueue. The same deadline race
protects this final recheck, so revocation or token expiry preempts both mutation
payloads and checkpoint-only progress frames.

The new regression permits the initial subscription and object checks, revokes
on the final subscription check, and proves the output channel remains empty
while a permission terminal is emitted. All ten stream security, ordering,
backpressure, and recovery tests plus strict pedantic Clippy pass.

## Warning: retention documentation described conditional validation

Disposition: fixed. `docs/ENVIRONMENT.md` now states that full-profile startup
always enforces the 24-hour broker retention minimum, including when CDC is
disabled, matching runtime validation and its regression.
