# p2-c009 — Admin UI scaffold

## Phase
phase-2-generated-sdks

## Depends on
p2-c008 (entity-management adapter must exist before the UI imports it)

## Directory
`admin-ui/`

## What this change does

Scaffolds the `admin-ui/` React 19 + Vite 7 application defined in CLAUDE.md.
This is the foundational scaffold only — no feature pages, no data. Establishes
the shell so Phase 3 can add feature modules without replatforming.

### Stack

| Concern | Choice |
|---------|---------|
| Framework | React 19 |
| Bundler | Vite 7 |
| Language | TypeScript (TSX everywhere) |
| UI primitives | shadcn-ui + Base UI (latest) |
| Routing | TanStack Router (file-based) |
| State | Zustand (client state) |
| Styles | Tailwind CSS v4 |

### Directory structure produced

```
admin-ui/
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.ts
├── index.html
└── src/
    ├── main.tsx                    # React 19 createRoot
    ├── core/
    │   ├── router.tsx              # TanStack Router root
    │   └── layout/
    │       ├── AppShell.tsx        # nav + sidebar + outlet
    │       └── AppShell.css
    ├── features/                   # empty — Phase 3 fills in
    ├── shared/
    │   └── ui/                     # shadcn-ui generated components go here
    └── infrastructure/
        └── sdk-client.ts           # SpineClient + RealtimeAdapter singleton
```

### Design constraints from CLAUDE.md

- Components render; hooks coordinate; stores own state; services call APIs.
- No component fetches data directly.
- No `any` types in TypeScript.
- TSX everywhere.

### Design quality requirement

The app shell must demonstrate at least 4 of the 10 required qualities from
`web/design-quality.md`. Recommended targets for the shell:

1. Clear hierarchy through scale contrast (nav vs content)
2. Intentional rhythm in spacing
3. Depth through surface layering (sidebar vs main)
4. Hover/focus/active states on nav items

Dark luxury or editorial direction is recommended. Do NOT use default
Tailwind/shadcn template output unstyled.

## Exit criteria

- `pnpm install` exits 0 from `admin-ui/`
- `pnpm typecheck` exits 0 (zero TypeScript errors)
- `pnpm build` exits 0 (Vite production build)
- `pnpm dev` starts without errors (visual verification not required for CI gate)
- App shell renders a nav + content area — visually intentional, not default template
