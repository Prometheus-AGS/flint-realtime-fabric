# p16-c013 — Bind remaining 5 proto services in SDKs + gateway

## Status: DONE

## Goal
G3.4 (H7) — phase-16-production-hardening

## Problem
Real SDKs (TS/Go/C#) wrap only SpineService; the gateway registers only AgentService server-side. Sync, agent, signal, entity, and authz are unreachable through any SDK.

## Solution
1. Bind sync/agent/signal/entity/authz clients in TS/Go/C#
2. Register missing servers in the gateway

## Files Changed
- `sdks/ts/`
- `sdks/go/`
- `sdks/csharp/`
- `crates/frf-gateway/src/`

## Risk
MEDIUM — mechanical per service once transport exists.
