# PGlite hard-reload stabilization receipt

Captured: 2026-09-17

Verdict: **PASS**

## Source binding

- Repository: `prometheus-entity-sync`
- Branch: `codex/production-readiness-integration`
- Base commit: `949cc53`
- Candidate diff SHA-256: `a2c03dfca18947680422a3142368ea6e215e2f29bb1119f1badae712592d096d`
- Final commit: `29e6cff`

## Change

The prototype now hosts each durable PGlite database through PGlite's supported
multi-tab worker instead of opening IndexedDB directly in every page. The
database key and durable contents remain unchanged. Vite emits an ES module
worker as required by PGlite's documented worker integration.

PGlite 0.5.8 documentation and installed type definitions establish that it
supports one database connection and provides `PGliteWorker` to elect one owner
across tabs. Closing or reloading a tab releases its worker ownership so another
worker can take over the same IndexedDB database.

## Local full-flow observations

- Production TypeScript/Vite build: PASS.
- Initial worker-backed connection: 10,075 ms, including first WASM startup.
- Hard reload and reopen of the same durable database: 2,870 ms.
- Concurrent second tab using the same database: 2,482 ms.
- Both tabs observed the same canonical row and its subsequent delete.
- Three additional leader-loss reloads reopened in 2,872 ms, 2,912 ms and
  2,916 ms; local rows were usable and the outbox was empty on every run.
- The earlier observed reopen delay was approximately 30 seconds.

The campaign used the running PostgreSQL, ElectricSQL, Fabric, Forge and auth
stack. The created row was deleted through the normal durable outbox path, and
both connected pages observed canonical deletion before cleanup.

## Commands

```text
pnpm --filter forge-electric-prototype build
Playwright browser campaign against http://127.0.0.1:5174
git diff --check
```

No isolated unit test or CI result is used as evidence.
