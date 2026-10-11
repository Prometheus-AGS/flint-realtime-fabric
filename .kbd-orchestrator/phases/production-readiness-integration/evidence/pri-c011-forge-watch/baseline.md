# c011 Forge watch baseline

Captured: 2026-09-16

## Owning checkout

- Repository: `git@github.com:Know-Me-Tools/flint-forge.git`
- Branch created for this phase: `codex/production-readiness-integration`
- Base revision: `dc313be3a044c65b05d845a9c34350bf5ca3ca3e`
- Base commit: `fix(postgres18): document + default pg_net.database_name=flint (#38)`
- Base working tree: clean
- Unstaged diff SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- Staged diff SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`
- Untracked-path-list SHA-256: `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`

## Dependency receipts

### pri-c005-authority-lifetime

- Fabric commit: `ec142e863944224f75f8111281ea15cf887e4dad`
- Receipt SHA-256: `618b43a8dcf46004028df24687f8ea4df3d5decab43dbe074d8905afe71db45e`
- Runtime receipt SHA-256: `96115a1b404dd8e7ef618aa0f4b3571d6093db33f97d9257fa52163b627259a4`
- Recorded result: PASS, seven owned local runtime scenarios, no CI evidence.

### pri-c010-type-watch

- Fabric commit: `52a655cbe8f8f701e6d56d50457a55b9b33a9a95`
- Remote branch: `origin/codex/production-readiness-integration`
- Receipt SHA-256: `59e6f69ee5afa433835b3ba06697543a8f6ae8e7e3a8d18922914a2fc948bf0a`
- Runtime receipt SHA-256: `13c7b0831053c1d95a73b8110822183ed48bcdf5cf4f820145d77d4813361e31`
- Recorded result: PASS, six owned local runtime scenarios plus focused local Rust gates; adversarial review round 23 PASS with no findings.

## Initial implementation findings

- `fdb-realtime::FabricChangeSource` still returns `StreamError::Unavailable`
  instead of invoking `flint.v2.EntityService.WatchEntityType`.
- Forge already retains the verified raw bearer in `RlsContext`, so it can
  forward the caller identity to Fabric without inventing another identity
  channel.
- `Quarry::subscribe_rls_filtered` currently uses the RLS query as a Boolean
  visibility check and forwards the original event payload. It must instead
  deliver the returned RLS projection so hidden columns never cross GraphQL.
- Delete events currently have no post-image, so the existing primary-key
  reconstruction drops them. Fabric v2 provides an authorized, key-only delete
  invalidation; Forge must preserve only those key columns in the GraphQL event.
- The Fabric v2 client is generated in `frf-proto` at the pinned c010 revision.
  The Forge dependency must be revision-pinned so an independent checkout
  reproduces the same wire contract.
