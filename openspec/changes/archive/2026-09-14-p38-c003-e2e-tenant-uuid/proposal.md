# p38-c003 — Fix the non-UUID tenantId in the E2E smoke script

## Summary

`tests/e2e/smoke_test.sh` publishes with a `tenantId` that is not a UUID, so the gateway
rejects every publish at the boundary. The script cannot pass today regardless of any other
fix in this phase.

## Evidence

`tests/e2e/smoke_test.sh:48` sends:

```
"tenantId": "e2e-tenant",
```

`crates/frf-gateway/src/grpc_service.rs:64-68` parses it strictly:

```rust
fn parse_tenant_id(s: &str) -> Result<TenantId, Status> {
    uuid::Uuid::parse_str(s)
        .map(TenantId::from_uuid)
        .map_err(|_| Status::invalid_argument(format!("invalid tenant_id UUID: {s}")))
}
```

`"e2e-tenant"` is not parseable as a UUID.

> **CORRECTION (T1, 2026-09-14).** An earlier draft of this section claimed "the sibling
> TS/Go/C# clients already use a valid UUID" for the tenant. That is **false**. A search for
> `tenant` in `tests/e2e/ts/smoke.ts`, `go/main.go` and `csharp/Smoke.cs` returns **nothing**
> — none of them sends a `tenantId` at all. What they share is `FRF_CHANNEL_ID`
> (`smoke.ts:14`), which is a **channel** UUID, a different field. I conflated the two.
>
> The defect is unchanged and still real; only my stated supporting evidence was wrong.
> The correct framing: `smoke_test.sh` is the **only** E2E client that sends a `tenantId`,
> so there is no sibling precedent to match. The precedent to follow is the script's own
> env-var convention at line 17:
> `CHANNEL="${FRF_CHANNEL_ID:-00000000-0000-0000-0000-000000000001}"`.

## Why this is independent

It touches only `tests/e2e/smoke_test.sh`. No overlap with c001 (compose/broker), c004
(gateway crate), c005 (root docs) or c006 (dead code).

## Scope

Use a valid UUID for the tenant — the fixture tenant `00000000-0000-0000-0000-000000000001`
already used by `CDC_TENANT_ID` in `compose.yml:29` — or read it from an env var with that
default, matching how the sibling clients handle `FRF_CHANNEL_ID`.

## Non-goals

- Making the whole E2E suite pass. A full green run may still depend on c001; this change
  fixes only the boundary rejection.

## Files

`tests/e2e/smoke_test.sh`.

## Outcome (2026-09-14)

Three hunks, following the script's own `FRF_CHANNEL_ID` convention rather than inlining a
literal:

1. `FRF_TENANT_ID` added to the "Environment variables:" doc block.
2. `TENANT="${FRF_TENANT_ID:-00000000-0000-0000-0000-000000000001}"` beside the existing
   `CHANNEL=` assignment, with a comment stating *why* it must be a UUID.
3. `\"tenantId\": \"e2e-tenant\"` → `\"tenantId\": \"${TENANT}\"`.

`shellcheck` was clean before the edit and remains clean after (exit 0), so no warning is
attributable to this change; `bash -n` passes.

The default value was cross-checked rather than assumed: `Uuid::from_u128(1)` — the
gateway's fixture tenant at `main.rs:99` — renders as exactly
`00000000-0000-0000-0000-000000000001`, the same value as `CDC_TENANT_ID` in
`compose.yml:29`.

### Verification status: DONE, NOT PROVEN

**Statically proven.** `uuid::Uuid::parse_str`, the exact call `parse_tenant_id` wraps:

| Input | Result |
|---|---|
| `e2e-tenant` | **REJECT** — "badly formed hexadecimal UUID string" |
| `00000000-0000-0000-0000-000000000001` | **ACCEPT** |

The script now emits the accepted value, so the `invalid_argument` rejection at the gateway
boundary is eliminated by construction.

**Not live-verified.** The Docker daemon (OrbStack) stopped mid-session and cannot be
started from the agent environment, so no gateway was available and no actual
`POST /flint.v1.SpineService/Publish` was issued. A full green E2E run additionally depends
on the rest of the stack, and remains unattempted.

This is deliberately *not* claimed as equivalent to `p38-c001`, which earned a live
integration run before it archived.
