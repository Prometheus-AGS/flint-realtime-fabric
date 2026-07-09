# p19-c003 — re-affirm the Dart async-transport deferral

## Why

The Dart async transport surface (`FrfTransport.connect` / `subscribe`) is deferred because
`uniffi-bindgen-dart 0.1.3` emits broken async/callback Dart (p18-c009 documented this and
shipped a compile-valid `FrfTransport` shim + a post-generation patch). Phase-19 (G5.1)
asks that this deferral be revisited: if a fixed generator exists, regenerate and remove
the c009 patch; if not, re-affirm the deferral with a **dated** upstream check so the
deferral note does not silently rot into a stale "coming soon."

## Finding (dated check — 2026-07-07)

The installed generator is `uniffi-bindgen-dart v0.1.3` (confirmed via `cargo install
--list`). I checked crates.io and pub.dev for a newer release on 2026-07-07; **no newer
async/callback-fixing version was found**. The async transport codegen remains broken
upstream. Therefore the deferral **stands** — regenerating today would reproduce the same
broken `frf.dart`, and there is nothing to remove the c009 patch against.

## What Changes

Documentation only — no code changes (per the assessment: writing async code today would
be pretending the generator is fixed).

1. **`sdks/dart/GENERATED.md`** — add a dated "Deferral re-affirmed" note recording the
   2026-07-07 check (installed 0.1.3; no newer async-fixing release found) and the exact
   follow-up trigger: when a fixed `uniffi-bindgen-dart` ships, regenerate via
   `build_dart.sh` and **remove the c009 post-generation patch**.
2. **`docs/SECURITY.md` §6** — keep the Dart async-transport row as re-affirmed deferred,
   pointing at the dated GENERATED.md note.

## Non-goals

- Implementing the async transport (blocked upstream — this is the whole point of the defer).
- A full hand-lowering of the async ABI (out of scope; the shim + patch are the interim).

## Impact

- Affected: `sdks/dart/GENERATED.md`, `docs/SECURITY.md`.
- No code or behavior change; the deferral is now dated and its removal trigger is explicit.
