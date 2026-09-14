# Tasks — p38-c003-e2e-tenant-uuid

> **DONE** = the script sends a valid UUID and the gateway accepts it.
> **PROVEN** = observed against a running gateway.

- [x] T1: Confirm `smoke_test.sh:48` still sends `"tenantId": "e2e-tenant"` and that
      `parse_tenant_id` still rejects non-UUIDs.
- [x] T2: Replace it with the fixture tenant UUID, or an env var defaulting to it —
      match the `FRF_CHANNEL_ID` pattern the sibling clients already use.
- [x] T3: Verify the publish is no longer rejected with `invalid_argument`. Note explicitly
      whether a full green run is still blocked by c001.
      **STATUS (2026-09-14): STATICALLY PROVEN, NOT LIVE-VERIFIED.** The Docker daemon
      (OrbStack) stopped mid-session and cannot be started from the agent environment, so no
      gateway could be stood up. What was proven: `uuid::Uuid::parse_str` — the exact call
      `parse_tenant_id` wraps — REJECTS `e2e-tenant` ("badly formed hexadecimal UUID string")
      and ACCEPTS `00000000-0000-0000-0000-000000000001`, which is what the script now emits.
      The boundary rejection is eliminated by construction. What was NOT done: an actual
      `POST /flint.v1.SpineService/Publish` against a running gateway. Per this file's own
      DONE/PROVEN vocabulary, T3 is DONE but NOT PROVEN.
