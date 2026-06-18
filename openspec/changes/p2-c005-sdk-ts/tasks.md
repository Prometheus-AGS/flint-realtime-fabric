# Tasks — p2-c005 sdk-ts

- [ ] **T1** Create `sdks/ts/package.json`
  - Name: `@prometheusags/frf-sdk`
  - Type: `"module"`
  - Exports: `"."` → `./dist/index.js` (ESM) + `./dist/index.cjs` (CJS)
  - Dependencies: `@bufbuild/protobuf@^2.5.2`, `@connectrpc/connect@^2.0.4`, `@connectrpc/connect-web@^2.0.4`
  - DevDependencies: `typescript@^5.8.3`, `@types/node@^22.0.0`
  - Scripts: `build: tsc`, `typecheck: tsc --noEmit`
  - Verification: file is valid JSON

- [ ] **T2** Create `sdks/ts/tsconfig.json`
  - `target: ES2022`, `module: NodeNext`, `moduleResolution: NodeNext`
  - `strict: true`, `noUncheckedIndexedAccess: true`
  - `outDir: ./dist`, `rootDir: ./src`, `declaration: true`, `declarationMap: true`
  - Verification: `pnpm tsc --showConfig` exits 0

- [ ] **T3** Verify generated stubs exist
  - Prerequisite: p2-c003 must have run `buf generate`
  - Verification: `ls sdks/ts/src/gen/flint/v1/*.ts` lists files

- [ ] **T4** Create `sdks/ts/src/client.ts`
  - `SpineClient` class with `static create(baseUrl: string)`, `publish(...)`, `subscribe(...)` using generated Connect-ES client
  - No `any` types; explicit return types on all public methods
  - Verification: file has no TypeScript errors (`pnpm tsc --noEmit`)

- [ ] **T5** Create `sdks/ts/src/index.ts`
  - Re-export `SpineClient` from `./client.js`
  - Re-export generated types from `./gen/flint/v1/envelope_pb.js` and relevant pb files
  - Verification: `pnpm tsc --noEmit` exits 0

- [ ] **T6** Install and build
  - `pnpm install` from `sdks/ts/`
  - `pnpm build`
  - Verification: `dist/index.js` and `dist/index.d.ts` exist; build exits 0
