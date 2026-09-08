# Flint Realtime Fabric documentation site

Docusaurus site published to GitHub Pages by `.github/workflows/docs-pages.yml`.

Built with the `build-branded-docusaurus` skill
(`scripts/scaffold.sh`), which produces a standard `create-docusaurus`
project using **npm**. It is deliberately **not** a member of the repo's pnpm
workspace — see "Why npm, not pnpm" below.

```bash
cd website
npm ci
npm run build     # sanitize, then docusaurus build
npm start         # dev server
```

## Verification

The skill's gate is the minimum bar, and it passes:

```bash
bash ~/.claude/skills/build-branded-docusaurus/scripts/verify.sh website
```

It runs `npm ci`, the sanitizer, a production build, and then greps the output
for machine-local paths and private key material. `npm run build` fails on a
broken link, because `onBrokenLinks` is `throw`.

## Why npm, not pnpm

An earlier attempt hand-assembled a Docusaurus site inside the repo's pnpm
workspace. It never produced a static build: the generated client-modules list
and route registry survived bundling as raw `require()` calls with absolute
paths instead of being inlined, and static generation then died on
`require.resolveWeak is not a function`, followed by Node trying to parse
Infima's CSS as JavaScript, and finally an unresolvable `@theme/DocsRoot`.

The scaffold the skill produces does not have that problem. **Keep this site on
npm with its own `package-lock.json`.** Adding `website` back to
`pnpm-workspace.yaml` is the specific change that reintroduces the failure.

## Structure

| Path | What it is |
|---|---|
| `docs/` | The documentation, 15 pages |
| `sidebars.ts` | Reading path: theory → guides → case studies → decisions → status |
| `src/pages/index.js` | Landing page |
| `src/css/custom.css` | Flat 2.0 base from the skill, plus the Ember brand layer |
| `scripts/sanitize.mjs` | From the skill; rejects private paths and secrets |
| `content-sources.yaml` | Source classification (public / normalize / private / excluded) |

## Design constraints

The skill's Flat 2.0 rule is enforced by the base CSS and this site keeps it:
**no visible borders, separator lines, gradients, or decorative shadows.**
Regions are distinguished by filled backgrounds and spacing. Keyboard focus
stays visible (`:focus-visible` outline in the base layer).

The brand layer appends Prometheus Ember tokens — `#E04E28` light, `#FF6A3D`
dark — over the skill's base without reintroducing any of the forbidden
treatments.

## Editorial standard

These pages distinguish three claims that documentation usually blurs:

- **Proven** — a test was executed, or a behaviour measured.
- **Built** — the code exists and compiles.
- **Gated off** — the code exists and is deliberately not enabled.

Where something is built but unproven, the page says so. `docs/status.md`
collects these, including four places where the repo's own docs currently
contradict each other.
