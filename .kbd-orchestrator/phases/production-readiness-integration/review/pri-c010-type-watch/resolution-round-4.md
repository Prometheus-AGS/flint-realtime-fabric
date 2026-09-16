# c010 adversarial review round 4 resolution

## Critical: alleged projection return-type mismatch

Disposition: rejected as a packet-context misread. The cited `Ok(false)` is
inside the local `prepared` async block, whose inferred type is
`Result<bool, PortError>`; the containing `apply` method maps that Boolean to a
`ProjectionApply` value afterward. Strict Clippy for every affected crate and
the full-stack projection/watch runs compile and execute this path.

## Critical: snapshot delivery can wait forever

Disposition: fixed. Initial snapshot frames now use a bounded awaited send.
When a client cannot drain the bounded queue within the configured authority
recheck interval, the producer terminates through the independent control path
with `Lagged { checkpoint_resumable: false }`. A focused regression fills the
queue without polling the receiver and proves the producer reaches terminal lag.

## Critical: subscription revocation can race mutation enqueue

Disposition: fixed. After object authorization succeeds, the producer now
rechecks the subscription tuple immediately before constructing and enqueueing
the protected mutation. Denial or an authorization error sets a permission
terminal. The output path already gives permission terminals priority over
buffered protected frames, so a visible revocation cannot be followed by an
older queued mutation.
