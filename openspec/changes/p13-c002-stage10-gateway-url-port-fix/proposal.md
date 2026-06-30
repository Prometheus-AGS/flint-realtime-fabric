# p13-c002 — Fix `GATEWAY_URL` port mismatch in Stage 10

## Summary

In `dagger/codegen.ts` Stage 10, the Dagger pipeline sets:
```
GATEWAY_URL=http://localhost:28080
```
but the gateway inside the DinD container network binds on port `:8080`
(internal). Port `28080` is the host-side mapping from `compose.yml`
(`28080:8080`). Inside the Dagger runner, `localhost:28080` is the
host-side port and may not be reachable; the internal Docker compose
network routes traffic to the gateway at `localhost:8080` (if using
host networking) or `gateway:8080` (via service name).

The healthz poll on line 299 already uses `http://localhost:8080/healthz`
(correct for internal container networking). `GATEWAY_URL` must match.

## File to change

- `dagger/codegen.ts` — line 303

## Specification

```typescript
// Change:
.withEnvVariable("GATEWAY_URL", "http://localhost:28080")
// To:
.withEnvVariable("GATEWAY_URL", "http://localhost:8080")
```

Port `8080` is the gateway's internal bind address (`BIND_ADDR: "0.0.0.0:8080"`
in compose.yml). Inside the Dagger runner with DinD, both `docker compose up`
and the Playwright E2E tests run in the same container, so `localhost:8080`
reaches the gateway.

## Acceptance criteria

1. Stage 10 Playwright tests resolve `GATEWAY_URL` to `http://localhost:8080`.
2. `layer2-publish.spec.ts` `gateway /healthz is reachable` test passes when
   run with `GATEWAY_URL=http://localhost:8080` and a running gateway.
3. No other reference to port `28080` remains in `dagger/codegen.ts`.
