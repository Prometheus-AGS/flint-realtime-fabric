# p16-c016 — Un-fork C# proto to frozen source

## Status: DONE

## Goal
G3.6 (#32) — phase-16-production-hardening

## Problem
The C# SDK commits its own copy of the proto files, forking the frozen proto-v1 source of truth (agent.proto already diverged).

## Solution
1. Generate C# from the frozen proto/ instead of a committed copy
2. Remove the forked proto copy

## Files Changed
- `sdks/csharp/`

## Risk
LOW — build/codegen change.
