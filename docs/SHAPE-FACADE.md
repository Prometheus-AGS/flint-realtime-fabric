# ADR-009 relational replication — the shape facade

The authorized read path between `ElectricSQL` and an ASO local SQL replica. ASO clients do
not talk to Electric directly; they talk to `GET /v1/shape`, which derives the allowed
practice, rows and columns **on the server** and authorizes every request *and every
continuation*.

> **Status: not certified, disabled by default.** On 2026-09-08 a bounded local composition
> exercised real Kratos v26.2.0 sessions, Gate RS256 grants, FRF verification and Electric
> 1.8.0. It passed an initial exchange, a same-session continuation with a fresh Gate token,
> scope, projection, cross-identity handle and expired-handle denials. A client-only segment
> reached Gate and could reach neither FRF, Electric's service address nor its operator-loopback
> diagnostic. A later local campaign measured session expiry, logout and membership removal
> against the fixed 5,000 ms bound; all closed without protected output in under 1,750 ms and
> new requests denied. Deployment-specific topology proofs remain open. The materializer,
> checkpoint atomicity and PEM publication contract are not implemented.

## Shape

```
crates/frf-ports/src/shape_facade.rs      the ShapeFacade port + request/response types
crates/frf-app/src/shape/
  ├── mod.rs                              authz, grant lifetime and handle binding
  └── policy.rs                           server catalog and row/column constraints
crates/frf-shape-electric/                one-port HTTP adapter; no authz dependency
  ├── client.rs                           the live Electric HTTP exchange
  └── facade.rs                           ShapeFacade impl over that client
crates/frf-gateway/src/routes/shape.rs    GET /v1/shape interface
crates/frf-gateway/src/main.rs            concrete dependency composition
```

The adapter implements exactly one port and holds no authorization dependency. `frf-app` owns
the use case that combines server policy, authorization and handle binding. `frf-gateway`
verifies identity and supplies concrete dependencies at the composition root.

## How widening is prevented

Three defences, all structural rather than conventional:

1. **Columns are never client-supplied.** They come from the policy entry alone. The request
   vocabulary has no way to name a column.
2. **Parameters are allow-listed by name, and an unknown key is an error** — not an ignored
   field. A caller that believes it narrowed must never silently receive a broader set.
3. **The scope predicate is server-composed and leads the filter.** Client terms are AND-ed
   onto it, so a filter can only ever intersect the authorized rows. Values are quoted as SQL
   literals; identifiers are validated; control characters are refused.

`AuthorizedShapeRequest` is the only input accepted by the adapter's `fetch`. The application
use case constructs it after the grant, projection, server policy and live authorization checks
pass. An unauthorized request is unrepresentable at the transport boundary.

## Authorization lifetime

The application use case checks authorization before and after Electric returns response metadata
on **every call**, including continuations. It then moves the upstream body into a capacity-one
producer that owns the original monotonic deadline, revalidates at intervals no greater than
750 ms and bounds each authorization call to 750 ms. The producer keeps those timers live while
the upstream stalls or the client applies backpressure. Deadline, denial, authority error or
consumer drop closes the producer and drops the upstream body. Axum receives the resulting
shell-neutral stream rather than a completed byte vector.

The gateway samples monotonic time before the request's sub-second Unix duration and passes the
pair into the application. FRF treats the integer JWT `exp` as the exact Unix-second boundary and
anchors its grant deadline to the earlier monotonic sample, so dispatch delay cannot extend the
grant. The response stream checks that deadline before and after receiver polling, independently
of the background producer's scheduling, so a ready buffered frame cannot cross an expired lease.

The full request and body-production lifetime retains the fixed 1,750 ms ceiling. Expiry before
response metadata returns produces `204 No Content`, which the Electric client treats as an empty
long-poll and follows with a new Gate request and fresh ASO replica-grant decision. Expiry after
metadata returns terminates the body stream. An output handle remains provisional until Axum polls
the completion marker behind the final ordered body frame. Normal completion then binds that handle
to the originating Kratos session, subject, practice, authorization revision, projection revision
and IDs, shape and grant expiry. Cancellation, a body error or consumer drop never commits the
provisional handle and removes the resuming grant's prior handle binding. A normal `304` with no new
handle preserves the prior binding. Gate may mint a new JWT ID for the same verified session without
invalidating its handle; another session or identity cannot reuse it.

ADR-009 rejects ADR-007's room-lifetime cache for ASO precisely because a grant held until
disconnect cannot meet a bounded revocation window; caching here would reintroduce that defect
on the relational lane. An authorization denial or error before or after fetch discards the
response and prevents handle binding. An authorization call or upstream response that never
returns reaches the same empty-response lease deadline.

The `verified-identity` FRF authorization backend validates relation names; it is not a fresh ASO
authority source. In that deployment, Gate's protected-response watchdog owns fresh ASO authority
revalidation and closes its FRF response on failure. Dropping that consumer aborts FRF's producer
and upstream body. FRF's independent 1,750 ms deadline remains effective if no Gate cancellation
arrives. Do not describe the local relation-name check by itself as an ASO grant decision.

The binding map is process-local. After an FRF restart, a client must start a new shape rather
than continue an unknown handle. Within one process, one Electric handle may retain bindings
for multiple authorized sessions without one session replacing another. Cross-process handle
state and multi-replica revocation remain work for the authorization-lifetime phase.

## Electric protocol

Snapshot, continuation, handle, offset and refetch semantics pass through unaltered:

| Client sends | Meaning |
|---|---|
| *(no cursor)* | Cold start — the facade sends Electric's `offset=-1` sentinel |
| `offset=now` | Start at Electric's current log position |
| `handle` + `offset` | Resume; both are opaque echoes of values Electric issued |

`live`, `cursor` and `If-None-Match` are also forwarded after authorization. A partial
continuation is rejected.

| Facade returns | Meaning |
|---|---|
| `electric-handle`, `electric-offset` | Resume position for the next request |
| `electric-up-to-date` | Initial snapshot complete — a cold-start view may now be shown as current |
| `409` + `electric-must-refetch` | Rebuild the replica generation; do not merge into stale rows |
| `204` | The bounded active lease elapsed without a deliverable response; reconnect through Gate |

Only Electric's protocol statuses `200`, `304` and `409` cross the adapter. FRF may originate
the Electric-client-compatible `204` above when its own active lease expires. Any other upstream
status is reduced to a private transport failure before its headers or body are read; the
gateway returns an empty `502` response instead of exposing upstream schema or query detail.
The gateway replaces any upstream cache policy with `Cache-Control: private, no-store` so an
authenticated shape body cannot be reused across browser sessions. Electric validators such
as `ETag` remain available to the active authorized caller.

## Configuration

Compile with `--features shape-facade`, then set **both**:

| Variable | Meaning |
|---|---|
| `SHAPE_ELECTRIC_URL` | Electric server origin |
| `SHAPE_CATALOG_PATH` | Path to the JSON shape catalog |
| `SHAPE_TIMEOUT_SECS` | Upstream timeout (default 30) |
| `GATEWAY_PROFILE` | Use `shape-only` for the bounded facade deployment; unrelated HTTP, gRPC, media and agent lanes are disabled |

The RA06 active-response lease is compiled at 1,750 ms and cannot be configured above the
recorded component budget. Gate's ASO deployment mints downstream replica tokens for at most
three seconds. This covers the 1,750 ms FRF lease plus the recorded one-second allowance for
whole-second JWT expiry precision.

The `shape-only` profile refuses to start unless both of the first two variables are set. In
the full profile, omitting both leaves the optional lane unmounted; setting only one is an
invalid configuration. A half-configured deployment therefore cannot reach Electric.

### Catalog format

```json
{
  "prior_auth": {
    "table": "prior_auth_request",
    "columns": ["id", "status", "submitted_at"],
    "allowed_params": ["status"],
    "relation": "view",
    "object_namespace": "practice",
    "reference": false,
    "scope_column": "practice_id"
  }
}
```

`relation` and `object_namespace` form the Keto check: the subject must hold `relation` on
`{object_namespace}:{scope}`, where `scope` is the practice from the caller's **verified
claims** — never from the query string. A token carrying no scope is refused rather than
defaulted.

`scope_column` is required for practice-owned projections. A projection may omit it only by
declaring `"reference": true`; the loader rejects an unscoped projection without that explicit
declaration and rejects a reference projection that also names a scope column. The declaration
cannot make an arbitrary table global. Reference projections must also match FRF's compiled
registry exactly. The registry currently contains only shape `evidence_states`, table
`aso.evidence_states`, columns `key`, `label`, `meaning`, no client parameters, relation `view`
and object namespace `practice`. The catalog is configuration because ASO defines its clinical
shapes, while the small compiled registry is the independent approval boundary for data shared
across practices.

A worked example is at [`docs/examples/shape-catalog.example.json`](examples/shape-catalog.example.json).
**Its table and column names are illustrative only.** ASO's deployment catalog lives in the
application repository and is checked against its privacy-approved replica schema. Do not
deploy the illustrative catalog as-is.

## What is deliberately absent

- **No write path.** ADR-009: one feed owns each clinical entity type, and the FRF event,
  CRDT, agent and media lanes cannot write the same authoritative clinical rows.
- **No row logging.** Diagnostics are counts, durations, schema versions, anonymized
  correlation and error classes only. A rejected parameter names its *key*, never its value.
- **No materializer, checkpoint or PEM publication.** Those are the replica side of ADR-009
  and are not implemented here.

## Before enabling

Current evidence against ADR-009's criteria:

- **Passed locally:** malformed requests cannot widen access; every continuation repeats
  authorization; a real Gate token reaches the FRF verifier; scope, projection and
  cross-identity handle mismatches return no shape metadata; a live Electric initial and
  continuation exchange succeeds.
- **Passed locally:** an expiring handle initially returns `200`; after its three-second
  binding expires, a fresh valid grant cannot continue it and receives `403` with no Electric
  headers.
- **Passed locally:** the internal client segment reaches Gate and receives its session
  challenge; FRF and Electric have backend-only network membership, the client cannot resolve
  either service or reach the host diagnostic, and the operator can reach the diagnostic only
  through `127.0.0.1`.
- **Passed in deterministic tests:** a blocked Electric response and a blocked authorization
  provider both stop at the 1,750 ms lease with no protected body or handle; a post-fetch denial
  discards rows and refuses handle binding.
- **Passed in deterministic tests:** after the first frame of a throttled three-frame response,
  logout, membership change and grant expiry suppress the remaining frames, drop the upstream
  body, release provisional continuation state and deny a new request. The direct-Kratos case
  exercises the production boundary: Gate observes the revoked session and drops its FRF response
  consumer; FRF then drops the upstream body and releases the continuation. The assembled live
  Kratos/Gate/FRF proof remains part of the parent revocation campaign.
- **Passed in deterministic timing tests:** a 1,000 ms test lease cancelled an upstream stall and
  a backpressured response at 1,000 ms without a client poll. An unavailable authority cancelled
  at the first 750 ms revalidation; a pending authority cancelled at 1,500 ms after its bounded
  750 ms decision timeout. Consumer drop stopped upstream work at the observed 125 ms drop time.
  Normal frames were observed in order at 0, 100 and 200 ms, completed at 200 ms and committed a
  reusable continuation. These are controlled monotonic-clock measurements, not network latency.
- **Passed locally:** session expiry, logout and membership removal closed active responses in
  278.672 ms, 1,737.861 ms and 1,429.459 ms respectively after the authoritative event. New
  requests denied; an unavailable grant authority closed the response in 1,749.861 ms and
  returned no protected body or Electric headers; eight racing requests could not restore the
  removed membership.
- **Passed locally:** a committed synthetic `gate_affirmed_at` transition to null reached the
  continued Electric stream with only approved columns and no foreign-practice row.
- **Open:** repeat the topology proof for each deployment-specific network before release.
- Prove SQL/checkpoint crash consistency and atomic PEM publication.
