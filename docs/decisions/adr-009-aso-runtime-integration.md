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

> **Implementation status — 2026-09-08.** The `ShapeFacade` port and one-port
> `frf-shape-electric` HTTP adapter exist. `frf-app` owns the server shape catalog,
> parameter allow-listing, per-request/per-continuation authorization and handle binding;
> `frf-gateway` verifies identity and exposes `GET /v1/shape` behind the off-by-default
> `shape-facade` feature. A bounded local composition exercised real Kratos v26.2.0
> sessions, Gate RS256 grants, the FRF verifier and Electric 1.8.0. Initial exchange,
> same-session continuation with a fresh Gate token, and scope, projection,
> cross-identity handle and expired-handle denials passed. A client-only segment reached
> Gate while direct FRF, Electric service and operator-loopback access failed. **The lane remains
> uncertified and disabled by default.** Persisted row-transition, measured session or
> membership revocation, deployment-specific topology, materializer, checkpoint atomicity
> and PEM publication proofs remain open.
>
> **Streaming correction — 2026-09-12.** FRF now treats the Electric handle returned in response
> metadata as provisional. It becomes reusable only when the server body consumer observes the
> completion marker after the final ordered frame. Lease cancellation, upstream body failure or
> consumer drop discards that provisional state and removes the current grant's resuming binding;
> a normally completed `304` preserves its prior handle. Status and allowed protocol headers remain
> unchanged, while the gateway continues to replace upstream cache policy with `private, no-store`.
>
> **Correction 2026-09-06:** an earlier revision of this note said ASO "has not yet defined the
> privacy-approved replica schema (sequence step 1)". That was wrong. It exists in the
> `prior-auth` repo — `web/src/shared/sync/pglite-schema.ts` (five tables, PHI exclusions as
> assertable data, ADR-007, a test that fails on a sixth table) and
> `web/src/shared/sync/electric-shapes.ts` (base-table relations and a column projection that is
> the PHI boundary, both measured against a live stack on 2026-09-05). `practice_id` is already
> denormalized onto every synced row because an Electric shape WHERE clause cannot join. The
> error came from reading the ASO sequence table's "step 1" as "not done" without checking the
> repo. FRF's catalog conforms to that schema; the schema does not change to suit FRF. See
> `docs/architecture/frf-shape-facade-integration.md` in `prior-auth`. The Verification
> section records the bounded criteria that have passed and those still open; nothing here
> certifies the lane.

Provide an authorized HTTP shape facade between Electric and the ASO local
SQL replica. The facade exists; its materializer integration remains planned work.
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

> **RA06 implementation status — 2026-09-09.** The bound was fixed before runtime
> implementation at 5,000 ms from authoritative revocation commit or session expiry through
> the last protected byte. Gate now retains authoritative Kratos expiry and uses a monotonic
> generation to prevent an invalidated cache miss, authentication result or Redis backfill from
> publishing stale identity. FRF streams Electric frames through a bounded background producer
> that retains the original 1,750 ms request deadline, revalidates at most every 750 ms, bounds
> each local authorization call to 750 ms and drops the upstream body on deadline, denial, error
> or consumer drop. A timeout before response metadata returns an empty Electric-compatible `204`;
> cancellation after metadata terminates the body. The shell samples monotonic time before
> sub-second Unix time and passes both, so FRF anchors integer JWT expiry to its exact second
> boundary without adding dispatch delay. The response stream checks its monotonic deadline on
> both sides of receiver polling so a queued frame cannot beat a waking
> cancellation worker. Gate's ASO composition mints a
> token for at most three seconds and every reconnect resolves a fresh ASO replica grant. The
> extra second covers whole-second JWT expiry precision while FRF still closes each protected
> response after at most 1,750 ms. A local campaign measured active-response closure 278.672 ms
> after session expiry, 1,737.861 ms after logout and 1,429.459 ms after membership removal.
> New requests denied, stale-refill racers did not restore access, and an unavailable grant
> authority closed in 1,749.861 ms and exposed no protected body or Electric headers. The
> `verified-identity` FRF backend is only a relation-name check, so Gate's response watchdog remains
> the fresh ASO authority owner and its downstream closure triggers FRF producer cleanup. The lane remains uncertified until
> its materializer, checkpoint atomicity and deployment-specific proofs pass.

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
replication. The bounded local shape composition has proved malformed request
denial, authorization on continuation, real Gate-to-FRF verification, a live
Electric initial/continuation exchange, expired-handle denial under a refreshed
grant, a committed synthetic gate transition, and a client segment that cannot bypass Gate,
FRF or reach the loopback diagnostic. Deterministic tests prove the fixed response lease,
post-fetch denial and Gate cache-refill fence. A local campaign proved bounded session expiry,
logout and membership removal with fresh-request denial and fail-closed authority loss. Each
deployment-specific topology, native parity, production Kratos deployment and protected media
remain separate gates.
