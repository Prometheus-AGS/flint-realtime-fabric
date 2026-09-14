# Tasks — p2-c008 entity-management-adapter

- [ ] **T1** Create `sdks/entity-management/package.json`
  - Name: `@prometheusags/frf-entity-management`
  - Type: `"module"`
  - Dependencies: `@prometheusags/frf-sdk` (workspace:* or relative path)
  - DevDependencies: `typescript@^5.8.3`, `@types/node@^22.0.0`
  - Scripts: `build: tsc`, `typecheck: tsc --noEmit`
  - Verification: valid JSON; `pnpm install` exits 0

- [ ] **T2** Create `sdks/entity-management/tsconfig.json`
  - Extend from `../../sdks/ts/tsconfig.json` or copy equivalent settings
  - `strict: true`, `noUncheckedIndexedAccess: true`, `noImplicitAny: true`
  - `outDir: ./dist`, `rootDir: ./src`
  - Verification: `pnpm tsc --showConfig` exits 0

- [ ] **T3** Create `sdks/entity-management/src/types.ts`
  - `EntityRecord`: `{ id: string; tenantId: string; entityType: string; payload: Record<string, unknown> }`
  - `EntityQuery`: `{ entityType: string; tenantId: string }`
  - `EntityEvent`: `{ kind: 'insert' | 'update' | 'delete'; record: EntityRecord; lsn: bigint }`
  - Verification: no TypeScript errors

- [ ] **T4** Create `sdks/entity-management/src/adapter.ts`
  - `RealtimeAdapter` class with `constructor(client: SpineClient)`
  - `watchEntities(query: EntityQuery): AsyncIterable<EntityEvent>` — subscribes via `client.subscribe`, translates `EventEnvelope` → `EntityEvent`
  - `mutateEntity(record: EntityRecord): Promise<void>` — builds `EventEnvelope`, calls `client.publish`
  - No `any` types; explicit return type annotations on all public methods
  - Verification: `pnpm tsc --noEmit` exits 0

- [ ] **T5** Create `sdks/entity-management/src/index.ts`
  - Re-export `RealtimeAdapter` from `./adapter.js`
  - Re-export `EntityRecord`, `EntityQuery`, `EntityEvent` from `./types.js`
  - Verification: `pnpm tsc --noEmit` exits 0

- [ ] **T6** Build
  - `pnpm build` from `sdks/entity-management/`
  - Verification: `dist/index.js` and `dist/index.d.ts` exist; exits 0
