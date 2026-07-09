# Flint Key Contract in Realtime

`flint-realtime-fabric` accepts JWTs minted or verified by `flint-gate`.
Browser WebSocket endpoints pass the token as `?token=...`; HTTP and gRPC calls
use `Authorization: Bearer <jwt>`.

Verified JWTs now preserve Flint principal metadata:

- `role`
- `principal_type`
- `agent_id`
- `workflow_id`
- `scope`

The existing tenant and subject rules still apply:

- `tenant_id` is authoritative and comes from the verified token.
- `sub` is the authenticated subject used for per-event authorization.
- `service_role` is server-side only; browser clients should use `anon` or
  user/agent tokens.

Realtime does not mint these keys. Use `forge keygen init` for local project
initialization and `flint-gate` for production token signing/verification.
