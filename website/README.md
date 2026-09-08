# Flint Realtime Fabric documentation site

Docusaurus site published to GitHub Pages by `.github/workflows/docs-pages.yml`,
mirroring the prometheus-entity-management setup: SHA-pinned actions,
least-privilege permissions, serialized deploys, and a deploy job a pull request
cannot reach.

```bash
pnpm --filter @prometheusags/frf-docs start   # dev server
pnpm --filter @prometheusags/frf-docs build   # production build
```

## Open blocker: the production build fails locally

`pnpm run build` currently fails with:

```
TypeError: require.resolveWeak is not a function
```

**Status: unresolved.** The content and configuration are complete and
`tsc --noEmit` passes; the bundler cannot produce the static site on this
machine.

### What was ruled out

Each of these was tested and is **not** the cause:

- **Node version.** Fails identically on Node 24.16.0 and 26.5.0. PEM's site
  builds on both.
- **Mermaid.** Fails with the theme and its dependencies fully removed.
- **Bundler.** Fails under both webpack and Rspack (`future.faster`).
- **`rspackPersistentCache`**, with and without.
- **Duplicate React.** The workspace holds 19.2.7 (admin-ui) and 19.2.8 (this
  site), but each member resolves exactly one, and the failure persists after a
  clean reinstall.
- **Site content.** Fails with a single trivial doc and the default config.
- **Workspace layout.** Reproduces standalone, outside the pnpm workspace.
- **`@docusaurus/react-loadable`.** Adding it explicitly does not help; PEM does
  not resolve it either and still builds.
- **pnpm version.** Reproduces under both pnpm 10.32.1 and 11.25.0.

### The unexplained difference

`prometheus-entity-management/website` builds green with the same Docusaurus
3.10.2, the same React 19.2.8, and an almost identical config — including after
its `node_modules` is removed and reinstalled. A minimal Docusaurus site created
fresh here fails. The difference has not been isolated.

### What the failure actually is (established this session)

The generated `.docusaurus/client-modules.js` and route registry survive
bundling as **raw `require()` calls with absolute paths**, instead of being
inlined as bundler modules. Compare the emitted server bundles:

| | prometheus-entity-management | this site |
|---|---|---|
| `require.resolveWeak` in `server.bundle.js` | **0** | 1 |
| raw `require("/…/*.css")` | **0** | 1 |
| `prism-include-languages` reference | **0** | 1 |

PEM's bundler inlines all of it; this site's leaves it raw. Everything the SSG
then trips over is downstream of that one fact.

`@docusaurus/core/lib/ssg/ssgNodeRequire.js` builds the `require` handed to the
server bundle from Node's `createRequire()`, and copies only `resolve`, `cache`,
`extensions` and `main` onto it. It never sets `resolveWeak`, and Node has no
such function — so the moment a raw registry entry survives, SSG dies.

### Why patching `ssgNodeRequire` is not the fix

A `pnpm patch` adding `resolveWeak` (returning the id unchanged) does work, and
it moves the build forward through three successive layers:

1. `require.resolveWeak is not a function` → fixed by the shim
2. `Unexpected token ':'` — Node parsing Infima's CSS as JavaScript
3. `Cannot find package '@theme/prism-include-languages'`
4. **`Cannot find module '@theme/DocsRoot'`** — and this is where it stops

Layers 2 and 3 can be no-ops honestly: stylesheets and theme client-modules
(Prism registration, the nprogress bar) have no server-render meaning. But
`@theme/DocsRoot` is the page component itself. Stubbing it would render blank
pages, which is worse than failing. The patch is therefore recorded here and
deliberately **not** applied.

### Hypotheses tested and eliminated

Each was tested directly, not reasoned about:

- Node version (24.16.0 and 26.5.0 both fail; PEM builds on both)
- Mermaid theme and its dependencies (removed entirely)
- Bundler: webpack, Rspack, and no `future.faster` block at all
- Individual `faster` flags: `swcJsLoader`, `mdxCrossCompilerCache`,
  `rspackPersistentCache`
- `@swc/core` postinstall enabled vs. disabled
- Duplicate React copies, and stale patched `@docusaurus/core` copies
  (clean `rm -rf node_modules` + `pnpm install --force`)
- Site content (fails with a single trivial doc)
- Custom `index.tsx`, `onDuplicateRoutes`, `additionalLanguages`
- Fontsource `@import` in CSS vs. PEM's `headTags` approach
- `@docusaurus/react-loadable` added explicitly
- pnpm 10.32.1 and 11.25.0
- Workspace layout — reproduces standalone, outside the pnpm workspace
- `@docusaurus/theme-classic` resolution (neither site resolves it; PEM builds)

### Suggested next step

The question to answer is narrow: **why does this site's bundler leave the
generated client-modules and route registry as raw `require()` calls, when
PEM's inlines them?** Everything else follows from that.

Two things worth trying, in order:

1. Diff the two *fully-resolved* dependency trees — `pnpm list --depth Infinity`
   in each site — rather than the manifests, which are already known to match.
2. Dump the resolved bundler config from both sites (patch
   `@docusaurus/core/lib/webpack/base.js` to write `module.rules` and
   `resolve.alias` to a file) and diff them. If the `@theme/*` aliases or the
   CSS rule differ, that is the answer.

Do not spend further effort on `ssgNodeRequire`; layer 4 above shows that path
cannot reach a correct build.
