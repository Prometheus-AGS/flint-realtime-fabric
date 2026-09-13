# Shape client — `GET /v1/shape`

The authorized relational replication read path (ADR-009). A consumer names a
**shape id** and echoes an opaque cursor; rows, columns and tenant scope are
derived server-side from verified identity.

```ts
import { ShapeClient, ShapeGrantExpiredError } from "@prometheusags/frf-sdk";

const client = new ShapeClient({
  baseUrl: "https://gate.example",      // Gate. Never Electric's own URL.
  credentials: { kind: "cookie" },       // browser; or { kind: "bearer", token }
});

let cursor;
for (;;) {
  const frame = await client.fetchShape("cases", cursor);
  if (frame.mustRefetch) { cursor = undefined; /* discard local state */ continue; }
  apply(frame.messages);
  cursor = frame.cursor;
  if (frame.upToDate) break;
}
```

## Before you file a bug: two things that look like client failures

**The endpoint may not exist.** `/v1/shape` is mounted only when the gateway is
built with `--features shape-facade`, which is **off by default** — ADR-009
keeps the lane uncertified until measured revocation and deployment topology
proofs pass. Against a default gateway you get a router 404, which this client
surfaces as `UnknownShapeError`. That is indistinguishable from a genuine
catalog miss, and the fix is different: check the build flags first.

**The catalog is a deployment artifact.** Shape ids come from the server's
`SHAPE_CATALOG_PATH` JSON, not from this package. A shape id valid in one
deployment is a 404 in another.

## Why not `@electric-sql/client`

That library always sends `table`, `columns` and `where`. A catalog entry with
`allowed_params: []` rejects every one of them with 400. The request contract
here is deliberately narrower than that client can express:

| Client sends | Meaning |
|---|---|
| `shape` | the catalog entry id — not a table, not a query |
| `handle` + `offset` | opaque continuation, echoed verbatim, both or neither |
| `live`, `cursor` | Electric long-poll and cache-bust echoes |
| `If-None-Match` | conditional request |

Anything else is narrowing input, permitted only if the shape declares it.

## Outcomes

| Result | Meaning | Recoverable |
|---|---|---|
| `messages` | row frames, control frames removed | — |
| `upToDate` | caught up with the stream | — |
| `mustRefetch` | 409 or `electric-must-refetch`; history is gone | restart cold |
| `notModified` | 304 | — |
| `ShapeGrantExpiredError` | 401 | re-establish the session |
| `ShapeForbiddenError` | 403 — refused, or handle not yours | no |
| `UnknownShapeError` | 404 — catalog mismatch | no |
| `InvalidShapeRequestError` | 400 — half cursor, or disallowed param | no |
| `ShapeUpstreamError` | 502 — the facade's Electric exchange failed | retry with backoff |

Deletes are **returned**, not dropped. A replica that rebuilds on `mustRefetch`
may ignore them; a client with a removal path must not have that decision made
for it. Inspect `message.headers.operation`.

## Tests

`src/shape/client.test.ts` pins the wire contract against
`crates/frf-gateway/src/routes/shape.rs`. **No test runner is configured in this
package** — the tests are written for `vitest` but nothing executes them yet.
Until a runner is wired, they are documentation of intent, not verification.
