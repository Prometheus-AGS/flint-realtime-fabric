# ADR-009 relational replication — the shape facade

The authorized read path between `ElectricSQL` and an ASO local SQL replica. ASO clients do
not talk to Electric directly; they talk to `GET /v1/shape`, which derives the allowed
practice, rows and columns **on the server** and authorizes every request *and every
continuation*.

> **Status: not certified, disabled by default.** The policy, authorization and
> protocol-mapping logic are unit-tested (17 tests). The **live Electric exchange has not been
> run against a server**, and ASO has not finalized the privacy-approved replica schema
> (sequence step 1 of the ASO runtime architecture). The materializer, checkpoint atomicity
> and PEM publication contract are **not implemented**. Per ADR-009 this lane stays off until
> its verification criteria are proved.

## Shape

```
crates/frf-ports/src/shape_facade.rs      the ShapeFacade port + request/response types
crates/frf-shape-electric/                the adapter (one port, no authz dependency)
  ├── policy.rs                           shape catalog: tables, columns, allowed params
  ├── resolver.rs                         policy resolution + the Keto check
  ├── client.rs                           the live Electric HTTP exchange
  └── facade.rs                           ShapeFacade impl over that client
crates/frf-gateway/src/routes/shape.rs    GET /v1/shape — the composition point
```

The adapter implements exactly one port and holds no authorization dependency. Composition of
(shape × authz) happens only in `frf-gateway`, mirroring ADR-007's media-path split.

## How widening is prevented

Three defences, all structural rather than conventional:

1. **Columns are never client-supplied.** They come from the policy entry alone. The request
   vocabulary has no way to name a column.
2. **Parameters are allow-listed by name, and an unknown key is an error** — not an ignored
   field. A caller that believes it narrowed must never silently receive a broader set.
3. **The scope predicate is server-composed and leads the filter.** Client terms are AND-ed
   onto it, so a filter can only ever intersect the authorized rows. Values are quoted as SQL
   literals; identifiers are validated; control characters are refused.

`AuthorizedShapeRequest` — the only thing the adapter's `fetch` accepts — can be produced
solely by `ShapeResolver::resolve`, which runs the Keto check. An unauthorized request is
therefore not merely rejected; it is unrepresentable at the transport boundary.

## Authorization lifetime

The resolver checks on **every call**, including continuations. There is no cached grant.

ADR-009 rejects ADR-007's room-lifetime cache for ASO precisely because a grant held until
disconnect cannot meet a bounded revocation window; caching here would reintroduce that defect
on the relational lane. A check failure (not just a denial) also refuses the request —
fail-closed, so an unreachable Keto is not an open door.

## Electric protocol

Snapshot, continuation, handle, offset and refetch semantics pass through unaltered:

| Client sends | Meaning |
|---|---|
| *(no cursor)* | Cold start — the facade sends Electric's `offset=-1` sentinel |
| `handle` + `offset` | Resume; both are opaque echoes of values Electric issued |

| Facade returns | Meaning |
|---|---|
| `electric-handle`, `electric-offset` | Resume position for the next request |
| `electric-up-to-date` | Initial snapshot complete — a cold-start view may now be shown as current |
| `409` + `electric-must-refetch` | Rebuild the replica generation; do not merge into stale rows |

## Configuration

Compile with `--features shape-facade`, then set **both**:

| Variable | Meaning |
|---|---|
| `SHAPE_ELECTRIC_URL` | Electric server origin |
| `SHAPE_CATALOG_PATH` | Path to the JSON shape catalog |
| `SHAPE_SCOPE_COLUMN` | Practice scope column (default `practice_id`) |
| `SHAPE_TIMEOUT_SECS` | Upstream timeout (default 30) |

If either of the first two is unset the lane is disabled and no route is registered — a
half-configured deployment gets **no route** rather than one backed by an empty catalog, which
would reject every shape and look like a client bug instead of a deployment mistake.

### Catalog format

```json
{
  "prior_auth": {
    "table": "prior_auth_request",
    "columns": ["id", "status", "submitted_at"],
    "allowed_params": ["status"],
    "relation": "view",
    "object_namespace": "practice"
  }
}
```

`relation` and `object_namespace` form the Keto check: the subject must hold `relation` on
`{object_namespace}:{scope}`, where `scope` is the practice from the caller's **verified
claims** — never from the query string. A token carrying no scope is refused rather than
defaulted.

The catalog is configuration, not code, because the set of clinical shapes is ASO's to define.
FRF loads it rather than hardcoding clinical schema.

A worked example is at [`docs/examples/shape-catalog.example.json`](examples/shape-catalog.example.json).
**Its table and column names are illustrative only** — they are not ASO's privacy-approved
replica schema, which does not exist yet (sequence step 1). Do not deploy it as-is.

## What is deliberately absent

- **No write path.** ADR-009: one feed owns each clinical entity type, and the FRF event,
  CRDT, agent and media lanes cannot write the same authoritative clinical rows.
- **No row logging.** Diagnostics are counts, durations, schema versions, anonymized
  correlation and error classes only. A rejected parameter names its *key*, never its value.
- **No materializer, checkpoint or PEM publication.** Those are the replica side of ADR-009
  and are not implemented here.

## Before enabling

ADR-009's verification criteria, none of which are met:

- Prove malformed shape requests cannot widen access.
- Prove every continuation is authorized.
- Prove expiry/revocation stops delivery within the chosen bound.
- Prove SQL/checkpoint crash consistency and atomic PEM publication.
- Run the live Electric exchange against a real server.
