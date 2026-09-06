# ADR-009: ASO runtime integration and authorization lifetime

## Status

Accepted ASO deployment target — 2026-09-06. Implementation is not certified.

Partially supersedes [ADR-002](adr-002-agent-bus-tenant-isolation.md) for ASO
protected agent streams and qualifies [ADR-007](adr-007-media-path-authz.md) for
ASO protected media. Generic fabric decisions remain scoped to their own uses.

## Context

The ASO application runtime architecture integrates Forge, Gate, FRF, PEM and
self-hosted Kratos. Tenant isolation alone does not establish an ASO subject's
permission to receive another user's run output. A grant cached until disconnect
or room teardown cannot meet bounded session revocation. This decision records
the required integration boundary, not its completion.

**Source:** `docs/architecture/application-runtime-architecture.md` in the
`prior-auth` repository ("Application runtime architecture: browser and Tauri",
status: accepted target architecture, implementation not certified), as of
commit `6bf36c2` (2026-09-06). That repository is a separate checkout with no
shared remote, so this ADR deliberately cites it by repo, path and commit rather
than by a filesystem path — a relative link out of this repository resolves only
on a machine where both are cloned as siblings, and breaks in CI and for any
other clone.

## Decision

### Relational replication

Provide an authorized HTTP shape facade between Electric and the ASO local
SQL replica. This facade and its materializer integration are planned work.
Preserve Electric snapshot, continuation, handle, offset and refetch semantics.
Derive allowed practice, rows and columns on the server; authorize every
continuation and prevent client parameters from widening the approved shape.
Commit replica rows and resume checkpoint atomically, then publish a coherent
PEM entity/list revision. One feed owns each clinical entity type.

FRF event, CRDT, agent and media services are separate lanes. They cannot also
write the same authoritative clinical rows. Local-only records and embeddings
of local data are structurally excluded at the egress boundary. A tenant ID,
small schema or omitted text field alone does not prove a projection is safe.

### Identity and authorization lifetime

ASO uses self-hosted Kratos browser cookies and native opaque session tokens.
Gate verifies Kratos sessions, resolves authoritative ASO membership and mints
short-lived, audience-bound downstream JWTs that FRF can verify. An opaque
Kratos token is not that JWT. Native credentials remain in the trusted host;
browser credentials and proxy downstream tokens are not persisted in Zustand.
The standalone admin-UI OIDC proposal in ADR-004 does not require Hydra for ASO.

Specify and test a maximum authorization staleness bound, including Gate cache
TTL, downstream token expiry, open streams, disconnect propagation and client
locking. Session expiry or revoked membership must stop protected delivery
within that bound. A token refresh must revalidate identity and membership;
refreshing an old grant cannot extend it indefinitely.

ASO protected agent output requires subject/run visibility before delivery, in
addition to tenant isolation. Use current authorization checks or bounded,
revocable grants evaluated at egress. Do not wait for run audit persistence or
a Redis cache before enforcing this requirement. Existing tenant-only channels
must not carry protected ASO output until this boundary is implemented and tested.

Optional protected media also requires bounded authorization expiry/revocation
and removal from fan-out. ADR-007's room-lifetime cache alone is insufficient.
A remote policy request per RTP packet is not required; a locally enforced,
expiring grant with revocation handling can meet the contract. Protected ASO
media remains disabled until that behavior is proved.

ASO logout and identity/practice switching increment the runtime epoch, drain
or fence old streams/writes and destroy the old graph. Persist noncredential
`logoutPending` before revocation to prevent silent cookie restoration after a
reload. Offline protected rendering is locked by default; no offline grant is
enabled here. Signing and affirmation remain online-only through Gate,
AppServices and authoritative Postgres.

### Deployment scope

Loro document state, fabric internal databases and shared SFU sockets are not
choices of ASO clinical replica storage. Browser PGlite and a Tauri PGlite
baseline precede the preferred host-owned SQLite target, which requires native
materializer and parity proof. PEM is the normalized Zustand graph in both.
The Tauri Zustand plugin is optional non-sensitive shell coordination only.

## Consequences

This integration adds a shape facade, membership/revocation contracts and
subject/run egress controls. The uncomfortable limitation is that existing
fabric tenant isolation and cached media grants do not yet prove ASO protected
stream safety. Documenting these requirements does not enable those workloads.

## Verification

Before enabling each protected lane, prove malformed shape requests cannot
widen access, every continuation is authorized, subject/run visibility is
respected, and expiry/revocation stops delivery within the chosen bound. Test
account/practice switches with in-flight work and logout across reloads. Prove
SQL/checkpoint crash consistency and atomic PEM publication for relational
replication. Native parity, real Kratos deployment and protected media remain
separate release gates. No runtime tests are claimed by this documentation edit.
