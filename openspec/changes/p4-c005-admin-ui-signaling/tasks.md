# Tasks — p4-c005 admin-ui-signaling

- [ ] **T1** Add Connect-ES and frf-wasm dependencies to admin-ui
  - File: `admin-ui/package.json`
  - Add `"@connectrpc/connect": "1.6.1"`, `"@bufbuild/protobuf": "2.3.0"`
  - Add local `"frf-wasm": "file:../../sdks/ts/frf-wasm"` (wasm-pack output)
  - Run `pnpm install`
  - Verification: `pnpm typecheck` (tsc --noEmit) exits 0

- [ ] **T2** Create `signalingStore` Zustand store
  - File: `admin-ui/src/features/signaling/stores/signalingStore.ts`
  - State: `roomId: string | null`, `status: 'idle' | 'joining' | 'joined' | 'left'`,
    `sfuMode: 'hosted' | 'sovereign'`, `participants: string[]`
  - Actions: `joinRoom(roomId: string)`, `leaveRoom()`, `onSignalFrame(frame: SignalEnvelope)`
  - No `any` types; import generated proto types from `sdks/ts/`
  - Verification: `pnpm typecheck` exits 0

- [ ] **T3** Create `signalingService`
  - File: `admin-ui/src/features/signaling/services/signalingService.ts`
  - Wrap `createBidiStreamingClient(SignalService, transport)` from `@connectrpc/connect`
  - Export `openSignalStream(roomId: string): Promise<void>` that pumps frames to store
  - Transport configured from `admin-ui/src/infrastructure/` (Connect-ES transport setup)
  - Verification: `pnpm typecheck` exits 0

- [ ] **T4** Create `useSignalingStream` hook
  - File: `admin-ui/src/features/signaling/hooks/useSignalingStream.ts`
  - Opens stream on mount (when `roomId` set in store), closes on unmount
  - Calls `signalingService.openSignalStream`, writes frames to store
  - Verification: `pnpm typecheck` exits 0

- [ ] **T5** Create `SignalingPanel` component
  - File: `admin-ui/src/features/signaling/components/SignalingPanel.tsx`
  - Renders room status badge, participant list, ICE status indicator
  - Uses Base UI / shadcn-ui primitives (Badge, Card, Button)
  - Reads from `signalingStore` via hook — no direct API calls
  - Intentional design: hierarchy through scale contrast, designed hover/focus states
    (per web design-quality rules — must not look like a default shadcn template)
  - Verification: `pnpm typecheck` exits 0

- [ ] **T6** Create CRDT demo button component
  - File: `admin-ui/src/features/signaling/components/CrdtDemoButton.tsx`
  - Imports `crdt_apply_delta` from `frf-wasm` (dynamic import for WASM init)
  - On click: merges two hardcoded snapshots, displays result in a `<pre>` block
  - Handles WASM load error gracefully (shows error message, no crash)
  - Verification: `pnpm typecheck` exits 0

- [ ] **T7** Wire demo page into routing
  - File: `admin-ui/src/core/` (wherever React Router routes are defined)
  - Add route `/demo/signaling` → lazy-loaded `SignalingDemoPage`
  - `SignalingDemoPage`: renders `<SignalingPanel>` + `<CrdtDemoButton>` + `<EntityStream>`
  - Verification: `pnpm build` exits 0; no TypeScript errors

- [ ] **T8** Playwright E2E smoke: signaling page loads
  - File: `admin-ui/e2e/signaling.spec.ts`
  - Navigate to `/demo/signaling`, assert `<h1>` visible, assert CRDT demo button present
  - Verification: `pnpm exec playwright test signaling.spec.ts` exits 0 (or skip if no browser in CI)
