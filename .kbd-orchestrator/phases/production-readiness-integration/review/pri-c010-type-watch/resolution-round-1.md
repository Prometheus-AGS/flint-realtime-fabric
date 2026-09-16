# c010 adversarial review round 1 resolution

## Critical: duplicate entity service registration

Disposition: false positive, clarified in code. `EntityGrpcService` imports and
implements `frf_proto::fv1::entity_service_server::EntityService`, while
`EntityTypeGrpcService` imports and implements the separate
`frf_proto::fv2::entity_service_server::EntityService`. The runtime aliases them
as `V1EntityGrpcService` and `V2EntityTypeGrpcService`. Tonic registers distinct
fully qualified service names: `flint.v1.EntityService` and
`flint.v2.EntityService`. The existing v2 contract test proves both generated
namespaces coexist, and the production gateway binary compiles with both routes.

## Critical: fallback checkpoint secret

Disposition: fixed. The all-zero key was removed. The v2 watch use case and
service are constructed only from a configured checkpoint key, and tonic's
optional-service registration omits v2 when that configuration is absent.
CDC-enabled validation still requires a key of at least 32 bytes.

## Critical: missing completion evidence

Disposition: fixed. The final real-stack receipt is stored as
`evidence/pri-c010-type-watch/runtime-receipt.json`, its acceptance and quality
record is current, and task 5 is checked after the first-round findings were
resolved.

## Warning: terminal delivery can overtake queued frames

Disposition: fixed with a security-preserving ordering rule. Source and
serialization terminals drain queued frames first. Lag and permission terminals
preempt the queue; permission preemption prevents protected frames buffered
before observed revocation from crossing afterward. Focused regressions cover
both the ordered source-error case and revocation preemption, including the
closed-channel race found by the full application suite.
