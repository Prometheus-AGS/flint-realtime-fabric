# Goals — Phase 5: Agent Protocols + BossFang (LibreFang fork)

> Seeded from Phase 4 reflection · 2026-06-20
> Corrected: BossFang IS the LibreFang fork — same codebase, GQAdonis fork

## Context

**BossFang** is the project's fork of [LibreFang](https://github.com/librefang/librefang),
a ractor-based publish/consume actor framework. There is no separate "LibreFang consumer"
vs "BossFang publisher" — BossFang provides both roles (publisher actor and subscriber actor)
as a unified fork. The `frf-librefang` crate wraps/vendors BossFang as the spine actor bus.

## Goals

- Implement `frf-agentproto` crate: AG-UI / A2A / A2UI schemas + ContentBlock types
- Implement `frf-librefang` crate: BossFang actor bus (GQAdonis fork) — publish and consume actors
- Wire `frf-librefang` into `frf-gateway` as the internal actor bus replacing direct Iggy calls in use-cases
- Admin UI agent activity panel (agent stream events from AG-UI)
- Phase exit criterion: An AG-UI agent event flows through BossFang → browser WebSocket consumer

## Prerequisite Decisions (resolve at assess / plan kickoff)

- Confirm ractor version (latest stable)
- Confirm AG-UI spec version from upstream
- Confirm BossFang fork URL / crates.io path vs. git dependency
- Decide: single BossFang actor tree per workspace OR one per tenant
