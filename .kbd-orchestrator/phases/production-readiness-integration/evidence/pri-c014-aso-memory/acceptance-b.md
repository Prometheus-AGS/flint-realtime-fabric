# pri-c014 acceptance B — browser allocation explanation

Result: **Passed**  
Observed: 2026-09-17T14:18:18.911268+00:00  
Dataset: 16,203 synthetic rows  
Campaign: `browser-campaign.json`  
Campaign SHA-256: `fff597119fce523e633b7030138159f758fa622e4e520776693c0d658e8f31ac`

## Outcome

The current ASO materializer completed the authenticated PostgreSQL → Kratos →
Gate → Fabric → Electric → browser PGlite → PEM campaign in Google Chrome
152.0.7977.84. The isolated target renderer grew by 387,547,136
bytes against the 536,870,912-byte limit. Chromium JavaScript heap grew by
142,408,106 bytes against the 268,435,456-byte limit.
The remaining headroom is 149,323,776 RSS bytes and
126,027,350 heap bytes.

SQL and PEM graph counts matched for all seven tables, all shape checkpoints
were present at generation 4, all 13 rollback/restart/delete/owner/authority/final-state
checks passed, and all fixture/service/identity cleanup checks passed.

## Measurement method

Chrome ran as an isolated process tree with a unique user-data directory. The
coordinator sampled macOS `ps` RSS for the Chrome root and recursive children,
then selected the renderer with the greatest campaign-time growth. JavaScript
heap came from Chromium `performance.memory.usedJSHeapSize` with
`--enable-precise-memory-info`. The browser loaded the production
`openPGliteReplica`, migration ledger, FRF shape transport, chunk writer,
checkpoint store, committed projector, and replica runtime.

Repeated actual-browser passes on the evolving candidate remained below both
limits. Two strict aggregate-gated passes on the final 31-file source closure
also remained below both limits. The latest source-bound pass is authoritative;
earlier passing receipts are retained to expose run-to-run variation rather
than hide it.

The final renderer RSS headroom is only
149,323,776 bytes. This satisfies the frozen
acceptance threshold but leaves little operational margin and remains a release
risk for later capacity qualification.

## Stage-associated bounds

These values are boundary nets and upper bounds. They do not claim
allocator-specific attribution.

| Boundary | Browser heap | Target-renderer RSS | Interpretation |
|---|---:|---:|---|
| Baseline | 7,268,561 | 167,395,328 | Loaded campaign page before PGlite starts. |
| PGlite open | 149,676,667 | — | PGlite/WASM and the versioned base archive dominate browser heap growth. |
| Schema ready | 63,042,313 | — | Replica schema and ledger add -86,634,354 net heap bytes. |
| Empty graph/projector | 63,652,857 | — | Graph runtime and projector add 610,544 net heap bytes before rows. |
| Cold fold complete | 67,858,698 | 546,996,224 | Electric folding adds 4,205,841 net heap bytes; renderer growth through this interval is 379,600,896 bytes. |
| Revision published | 125,735,011 | — | Collection occurs during transactional SQL/checkpoint/readback/graph work, so the lower heap boundary is not a negative-allocation claim. |
| Campaign complete | 123,396,549 | 554,942,464 | The 7,946,240-byte RSS increase after the cold-fold stage is an upper bound for SQL/checkpoint/readback/graph publication and retained runtime work. |

`ReplicaWorkerOwner` does not create a second Worker or database. The current
browser implementation runs the materializer in one renderer. The separate
Node durability scenario proves owner disposal, replacement, and authority
fencing; its interval includes database and revision work and therefore remains
an upper bound rather than an owner-only allocation.

## Behavior preserved

The final campaign passed initial SQL/graph parity, complete checkpoints,
gate-summary publication, reference identities, cold rollback, captured-update
rollback, restart without a skipped commit, delete-triggered replacement,
disposed-owner fencing, replacement-owner continuation, and authority-failure
fencing. Final SQL/graph counts and checkpoint generation also matched. The
campaign content-hashed all 31 directly executed/imported source and
configuration inputs before import and after cleanup; both aggregate hashes
were `b71fc53df00686138ea1472cbff2ed01fa91d592b0d1761b6ae8e272dd70c3dd`.
No production adoption flag changed.

## Source binding

ASO HEAD: `d95242542060d1e19a6efe9143f57c015d6162f4`

| File | SHA-256 |
|---|---|
| `docker-compose.ra05.yaml` | `1815b800b226a0b356d903062f172d9377108194fa5bbf9f1b18cdd9687a50bc` |
| `docker-compose.yaml` | `6f642f9f561b78a46fae7f1f3863dcdcd633c9dd2730954a58ee87b1bbda7e8b` |
| `docker/flint-gate/config.ra05.yaml` | `347d3dd0d30f2221fbbe39237f967b0511e703ab5ecee8e327a87b78ff3dafa1` |
| `docker/frf/shape-catalog.json` | `7c50cc69ff87cfd5db56fc847c17ac035f97bd900ae71a7eee286fca286ce453` |
| `scripts/ra05-stack.sh` | `824377cc9561e71474da079219527a4b1846a02592653e5a5ec430a2fc7cf2f4` |
| `scripts/ra06c_campaign_config.py` | `cd99908ae918fb7a4d1eff48500aad329ac923ad477bb1f2bfb77e8706fa9b4e` |
| `scripts/test-authorized-shape-composition.py` | `bbac93420ab50c53624c3c06c9c27d61ef94ca2fa28dfeb4031cbe09346f09d7` |
| `scripts/test-ra11a-sync-conformance.py` | `b21d01815058e7307f30989b11ce35c3954e3910e50f47a2bc05158e6915261a` |
| `scripts/test-ra11c-browser-memory.py` | `10e3c049d5c4299e34bb1024377d06912a4fb5ff05b5f4de3ea4860fc1b24205` |
| `scripts/test-ra11c-materialization.py` | `be8af39283fe56e1fac1c384a755986df5e10712770f96ca02962730f2b9bdf1` |
| `web/package.json` | `c8c7a697a514ddc18246f35e2c12db5eaa07bfa2f63ab131034b42c1c8be6641` |
| `web/pnpm-lock.yaml` | `fbe014d8cde45225e4c8884edd5193ddcd387295f065fbbb27690fd0269c2547` |
| `web/public/pglite-base.tgz` | `f4818a049c2e017209433b00518760dbd8694d20c88fdc59b3bc2aea299f635f` |
| `web/ra11c-browser-memory.html` | `4a26bd7963b6efb023a4a5f152170a9e350e9737a18f982b97b449e17662dd72` |
| `web/src/shared/session-revocation-events.ts` | `e6c691d5b611aa9e8add31d3995d60529aa274b132e3ef67d357a93580ca6850` |
| `web/src/shared/sync/chunk-writer.ts` | `e3b25011f44d25b969759599c0abd90664ea873cc0996da454572f6c4815826e` |
| `web/src/shared/sync/electric-shapes.ts` | `830e2540b6e03d5356243a390c622b467fde5e637ecdceb5030afc492be9944c` |
| `web/src/shared/sync/frf-shape-transport.ts` | `49751868d6d78da375c1750e6093e2c27e39403b48152f3de1d62fc1ed9e176c` |
| `web/src/shared/sync/migration-ledger.ts` | `206d0e3ce46181653db0b1b5d2c675c7a7018ae8e773c1d4c0c7d9da95567bc4` |
| `web/src/shared/sync/pglite-bootstrap.ts` | `5d7cf6b33bfae8c0c6e74287950575c10741c2aa3e07dddca2b69045a799b0c6` |
| `web/src/shared/sync/pglite-schema.ts` | `7754586386bbfd29144e7cb47f9f188a41bd8c6f17aa688dce48d6f4a0bf433e` |
| `web/src/shared/sync/replica-browser-memory.ts` | `a8d17a9a591b08cce218f843621b35cf9b9788320b241a0eb98bb4f6e56a2208` |
| `web/src/shared/sync/replica-mounted-runtime.ts` | `7caaaa829f37e7f393bd3f56f9de015ef6503acdd5536cbca4e9f2f9d37756c0` |
| `web/src/shared/sync/replica-mounted.integration.test.ts` | `5e394e3e505b7f6cb9daf32ba8dd8c61a4398e94457325f18f42c52470c93849` |
| `web/src/shared/sync/replica-owner.ts` | `4352138b06c24f55e6cfad5a0ada89ba04636e0e9657c3ccb494d14af39ab670` |
| `web/src/shared/sync/replica-rebuild.ts` | `fe79cfc4573d9af144c20293e08e8dcb3e490a1c1ecd2988abd2d17273cdb76a` |
| `web/src/shared/sync/replica-runtime.ts` | `97a8180893f0039ecb2e23ca2a1b86ca5d3d6a2432d50f5ae0d430d808a5e5fb` |
| `web/src/shared/sync/replica-wiring.ts` | `ce52c79d4e2c113084913e9df34e1b2502b5c20838a0df748c22f1382e45484b` |
| `web/src/shared/sync/replica-worker-owner.ts` | `616a71b27e461954efb701dc18f686dce81e818edd5ef0ff2f23076f96063f5b` |
| `web/src/shared/sync/storage-policy.ts` | `74ebfba4eb1ad96efc032de39f3f3715ad5f861a0a776af3a6b2ed8b78bd58a0` |
| `web/vite.config.ts` | `89da9c620bb5d18f0b8eaf8d4fba7805bac56dcd23ffc87e0cce2e17ad01d8fd` |
