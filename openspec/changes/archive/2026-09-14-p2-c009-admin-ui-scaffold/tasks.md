# Tasks — p2-c009 admin-ui-scaffold

- [ ] **T1** Scaffold Vite + React 19 project
  - `pnpm create vite@latest admin-ui -- --template react-ts` (or equivalent)
  - Upgrade React to 19 if scaffolder pins an older version: `pnpm add react@19 react-dom@19`
  - Verification: `admin-ui/src/main.tsx` exists; `pnpm install` from `admin-ui/` exits 0

- [ ] **T2** Install and configure Tailwind CSS v4
  - `pnpm add -D tailwindcss@^4.0.0 @tailwindcss/vite`
  - Add `@tailwindcss/vite` plugin to `vite.config.ts`
  - Create `admin-ui/src/styles/tokens.css` with CSS custom properties for color, spacing, typography
  - Verification: `pnpm build` exits 0; no Tailwind config errors

- [ ] **T3** Configure strict TypeScript
  - `tsconfig.json`: `strict: true`, `noUncheckedIndexedAccess: true`, `noImplicitAny: true`
  - Add `pnpm typecheck` script: `tsc --noEmit`
  - Verification: `pnpm typecheck` exits 0 on scaffold

- [ ] **T4** Install shadcn-ui + Base UI
  - `pnpm dlx shadcn@latest init` (select Tailwind v4, TypeScript, src/shared/ui output)
  - `pnpm add @base-ui-components/react@latest`
  - Add at least: Button, Separator, Tooltip components from shadcn
  - Verification: `src/shared/ui/button.tsx` exists; `pnpm typecheck` still exits 0

- [ ] **T5** Install TanStack Router + Zustand
  - `pnpm add @tanstack/react-router zustand`
  - `pnpm add -D @tanstack/router-vite-plugin`
  - Add router plugin to `vite.config.ts`
  - Create `src/core/router.tsx` with a root route and `__root.tsx` layout
  - Verification: `pnpm build` exits 0

- [ ] **T6** Create `AppShell` layout component
  - `src/core/layout/AppShell.tsx`: collapsible sidebar + top nav + `<Outlet />`
  - Apply intentional styling (dark luxury or editorial direction):
    - Nav: scale contrast, depth via surface color
    - Items: hover/focus/active states (not default gray)
    - Spacing: intentional rhythm (not uniform padding)
  - Must satisfy ≥4 of the 10 design quality criteria from `web/design-quality.md`
  - Verification: `pnpm typecheck` exits 0; no `any` types in AppShell

- [ ] **T7** Create `src/infrastructure/sdk-client.ts`
  - Import `SpineClient` from `@prometheusags/frf-sdk`
  - Import `RealtimeAdapter` from `@prometheusags/frf-entity-management`
  - Export singleton: `export const spineClient = SpineClient.create(import.meta.env.VITE_GATEWAY_URL ?? 'http://localhost:3000')`
  - Export `export const realtimeAdapter = new RealtimeAdapter(spineClient)`
  - Verification: `pnpm typecheck` exits 0

- [ ] **T8** Production build gate
  - `pnpm build` from `admin-ui/`
  - Verification: exits 0; `dist/` directory created; no TypeScript errors during build
