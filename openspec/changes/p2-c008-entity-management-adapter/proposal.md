# p2-c008 — entity-management RealtimeAdapter

## Phase
phase-2-generated-sdks

## Depends on
p2-c005 (TS SDK must be built before the adapter can import it)

## Directory
`sdks/entity-management/`

## What this change does

The CLAUDE.md SDK strategy states:

> entity-management: Thin `RealtimeAdapter` on TS SDK — treat as a new SDK

This change scaffolds the `entity-management` package as a thin adapter layer
that wraps `@prometheusags/frf-sdk` with domain-specific entity operations.

### Package structure

```
sdks/entity-management/
├── package.json              # @prometheusags/frf-entity-management
├── tsconfig.json
└── src/
    ├── index.ts              # re-exports RealtimeAdapter, entity types
    ├── adapter.ts            # RealtimeAdapter class
    └── types.ts              # EntityRecord, EntityQuery, EntityEvent
```

### `RealtimeAdapter` surface

```ts
export class RealtimeAdapter {
  constructor(client: SpineClient)

  // Subscribe to real-time entity changes filtered by type and tenant
  watchEntities(query: EntityQuery): AsyncIterable<EntityEvent>

  // Publish an entity mutation event
  mutateEntity(record: EntityRecord): Promise<void>
}
```

### Design constraints

- `RealtimeAdapter` must NOT contain business logic — it translates
  `EntityRecord`/`EntityQuery` domain types to `EventEnvelope` wire types.
- No direct gateway calls — all network I/O flows through `SpineClient`.
- Zero `any` types in TypeScript.

## Exit criteria

- `pnpm install` exits 0 from `sdks/entity-management/`
- `pnpm tsc --noEmit` exits 0 (zero type errors)
- `pnpm build` exits 0
