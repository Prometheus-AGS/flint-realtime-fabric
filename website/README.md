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

### Suggested next step

Diff the two fully-resolved dependency trees (`pnpm list --depth Infinity` in
each site) rather than comparing manifests, and look for a transitive package
present in PEM's tree and absent here. `require.resolveWeak` is injected by the
bundler runtime, so the culprit is likely a package that changes how the module
registry is emitted.
