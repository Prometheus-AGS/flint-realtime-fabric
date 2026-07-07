# p15-c001 — Fix Dagger Stage 10 port references

## Status: PROPOSED

## Problem

The Stage 10 Dagger block in `dagger/codegen.ts` polls the gateway healthz
endpoint and passes `GATEWAY_URL` to Playwright using port `8080`. However,
`compose.yml` maps the gateway container port as `28080:8080` (host:container).

In the DinD environment the Dagger runner operates on the Docker host network.
The gateway is reachable at `localhost:28080`, not `localhost:8080`. As a
result:

1. The healthz poll loop (`curl -sf http://localhost:8080/healthz`) will time
   out after 60 seconds, causing Stage 10 to fail before any test runs.
2. `GATEWAY_URL=http://localhost:8080` passed to Playwright causes all
   Phase 4/5/6 Layer 2/3 integration tests to fail with connection refused.

## Solution

Change both references from port `8080` to port `28080` in the Stage 10 block
of `dagger/codegen.ts`:

1. Healthz poll: `http://localhost:8080/healthz` → `http://localhost:28080/healthz`
2. `GATEWAY_URL`: `http://localhost:8080` → `http://localhost:28080`

## Files Changed

- `dagger/codegen.ts`

## Risk

NONE — pure port number change. The compose port mapping is authoritative;
this change aligns the Dagger client with the actual host-exposed port.
