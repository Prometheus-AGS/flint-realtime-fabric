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

`"e2e-tenant"` is not parseable as a UUID. The sibling clients are unaffected —
`tests/e2e/ts/smoke.ts:14`, `go/main.go:28` and `csharp/Smoke.cs:11` all default
`FRF_CHANNEL_ID` to a valid UUID; it is specifically the shell script's hard-coded tenant
string that is malformed.

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
