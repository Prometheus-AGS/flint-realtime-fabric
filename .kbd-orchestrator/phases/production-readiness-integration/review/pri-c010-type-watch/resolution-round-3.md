# c010 adversarial review round 3 resolution

## Critical: checkpoint payload is readable

Disposition: fixed. Checkpoint payloads are now encrypted and authenticated with
AES-256-GCM. Each issuance uses a fresh 96-bit operating-system nonce, and the
outer version and generation are authenticated as associated data. The configured
secret is domain-separated and reduced to a 256-bit encryption key. Tests prove
the subject and source epoch do not appear in token bytes, identical payloads
produce different tokens, tampering fails, and scope/epoch/generation validation
still fails closed.

## Critical: revocation can race a ready frame

Disposition: fixed. The output stream prioritizes terminal changes, rechecks for
preempting permission/lag terminals after both immediate and awaited frame
receipt, and discards a received frame if revocation became visible before
delivery. Source and serialization terminals still drain frames queued before
the error. Focused ordering tests and the full resume/lag/revocation test pass.

## Critical: timestamp without time zone changes canonical type

Disposition: fixed. The frozen v2 protobuf has no distinct field for PostgreSQL
`timestamp without time zone`, so transport conversion now returns gRPC
`DATA_LOSS` instead of mapping it to `time_value`. A regression asserts the
fail-closed behavior.
