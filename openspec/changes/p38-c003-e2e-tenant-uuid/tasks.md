# Tasks — p38-c003-e2e-tenant-uuid

> **DONE** = the script sends a valid UUID and the gateway accepts it.
> **PROVEN** = observed against a running gateway.

- [ ] T1: Confirm `smoke_test.sh:48` still sends `"tenantId": "e2e-tenant"` and that
      `parse_tenant_id` still rejects non-UUIDs.
- [ ] T2: Replace it with the fixture tenant UUID, or an env var defaulting to it —
      match the `FRF_CHANNEL_ID` pattern the sibling clients already use.
- [ ] T3: Verify the publish is no longer rejected with `invalid_argument`. Note explicitly
      whether a full green run is still blocked by c001.
