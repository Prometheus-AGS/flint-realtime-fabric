# Dependency qualification — pri-c003

Observed: 2026-09-15. Local lockfiles and source revisions are the pinning
authority. Context7 supplied current API documentation; official repositories,
registries, changelogs and the RustSec database supplied maintenance/security
evidence. Documentation establishes API fit, not runtime fitness.

## Release-data dependencies

| Dependency | Resolved identity | API and compatibility evidence | Maintenance/security evidence | c003 verdict and owner |
|---|---|---|---|---|
| Apache Iggy client/server | fork crate `iggy 0.6.203`, commit `d34b9c96ad5a15334e06040d68fd7512beeba4c8`; Compose image currently `iggyrs/iggy:latest` | Pinned source supports explicit poll offsets and consumer offsets, but FRF currently exposes producer counters and leaves auto-commit behavior active | Upstream is active and preparing 0.9 with a new wire protocol; the fork pulls vulnerable `quinn-proto 0.11.14`, unsound `event-listener 5.4.1`, and affected `rustls 0.23.40` | **Conditional retain.** c007 must pin a compatible server digest, patch the dependency closure, and prove seek/ack/replay on this fork before release |
| PostgreSQL WAL parser | `pg_walstream 0.6.3`, checksum `54e3661e…d69d59a` | Installed source provides relation metadata, transaction/control events and applied-LSN support; FRF does not yet preserve those semantics | Repository is active; current tag is `v0.8.1` at `df60b02c…`. The pinned 0.6 line is behind and no project security policy was found | **Revise before production.** c008 must compare 0.6.3 to 0.8.1, select an exact version, and run compatibility/security and transaction recovery proofs. No silent upgrade |
| Ory Keto | deployment tag `oryd/keto:v0.12`; Fabric uses HTTP relation tuple APIs | Context7 confirms `/health/ready`, `/health/alive`, `/version`, individual checks and batch checks | Upstream latest is `v26.2.0`; official guidance says business-critical self-hosting needs an explicit support/risk decision. The current tag is not digest-pinned | **Retain protocol, deployment unqualified.** c004/c005 pin the chosen build/digest, migration state and support posture; fail readiness closed |
| Cedar | `cedar-policy 4.11.1`, checksum `716a5103…54e43` | Public `Authorizer`, request, policy-set and diagnostic APIs fit the in-process policy adapter; exact Cargo pins are recommended for deterministic behavior | Active; 4.11.2 fixes a cyclic hierarchy defect introduced in 4.11.0 and 4.12.0 is tagged. Cedar publishes security practices | **Patch required.** c005 must move to a compatible fixed exact release and re-run policy parity/negative tests before enabling Cedar in a release profile |
| Connect-ES / Protobuf-ES | lock resolves `@connectrpc/connect 1.7.0`, `connect-web 1.7.0`, `@bufbuild/protobuf 1.10.1` | v1 supports browser transport and streaming; v2 changes generated types and interceptor streaming shapes. A v2 proto namespace does not require Connect-ES v2 | Project is active, stable and semver-governed; latest is 2.2.0. The maintained-client window covers recent browsers/Node, but FRF must prove the selected v1 line | **Retain v1 toolchain.** c012 fixes ESM/CJS exports and proves packed browser/Node clients over the real gateway; upgrade only by a separate compatibility decision |
| PGlite | ASO lock resolves `@electric-sql/pglite 0.5.8`; base tarball SHA-256 `f4818a04…9f635f` | IndexedDB/filesystem persistence and worker ownership exist; minor upgrades require dump/import. API presence does not prove resource use | npm latest remains 0.5.8; repository active. Current ASO campaign measured 1,016,692,736 bytes incremental RSS | **Retain experimentally, release blocked.** c014 must meet the fixed browser budget on the exact package and fixture; c015 then proves protected delivery |
| PEM packages | ASO pins core/react `4.0.3-ra11c.1.g071b9e5.sbb3dc7729aa7`; PEM source is `071b9e5…` | Current core/react packages are ESM; the accepted first release requires Fabric SDK ESM/CJS separately and PEM browser graph application | Source-bound ASO pin and tarball provenance exist; publication remains distinct from implementation | **Retain.** c012 must install packed artifacts and prove canonical SQL-to-graph/list delivery without sibling-source imports |

## Deferred-profile dependencies retained behind separate gates

| Dependency | Resolved identity | Evidence and boundary | Verdict |
|---|---|---|---|
| SurrealDB Rust SDK/store | `surrealdb 3.1.5`, checksum `81ee3110…fcaab`; Compose uses mutable `latest` | Docs describe version pinning/backups. 3.1.5 is the patch floor for multiple 2026 advisories, but its HTTP RPC client has a reported attached-session leak; its closure includes vulnerable `ammonia`, `crossbeam-epoch`, `h2`, `rsa`, `rustls` and unsound `anyhow`/`event-listener` | **Blocked for a durable profile.** c004/c013 must pin server/client identities, prefer/prove a safe transport, clear applicable audit findings, and prove backup/restore |
| redb | `redb 4.1.0`, checksum `8e925444…2e839`; upstream tag 4.3.0 | Immediate durability is the default; two-phase commit plus quick repair trades commit cost for fast recovery | Active, exact pin present; no direct RustSec finding in the 2026-09-14 database | **Retain.** c013 must use a file-backed store, set accepted durability, and prove restart/repair |
| Loro | `loro 1.13.1`, checksum `7bd1b63c…301c7` | Snapshot/update export and version-based incremental import fit CRDT recovery | The dependency path includes unmaintained and unsound `im 15.1.0` and `sized-chunks 0.6.5` with no fixed version on that line | **Blocked.** c013 must adopt a compatible dependency line without those advisories or revise the CRDT proposal |
| LiveKit Rust APIs | `livekit-api 0.5.2`, checksum `394e625d…a1fa`; latest observed release 0.6.1 | Server APIs manage rooms/permissions; actual reliable data receive/publish needs the realtime SDK and room-event loop, which FRF does not compose | Active releases; current closure includes affected `rsa`, `rustls`, `h2` and `anyhow` | **Hosted profile blocked.** c017 must select exact server and Rust SDK versions, clear applicable audit findings, and prove real participant/data transport |
| str0m | `str0m 0.21.0`, checksum `fed3d929…b025` | Sans-I/O engine delegates UDP/socket/timer work to Fabric; app must drain output and feed receive/timeouts. It is not an SFU deployment by itself | Repository active with recent security and correctness fixes in changelog; no direct RustSec match in the current lock audit | **Retain.** c020 proves actual decoded frames, ICE/TURN, restart/cancellation and capacity on the exact pin |
| Matrix protocol | raw reqwest adapter; protocol target is released Matrix v1.19 | `/sync` requires durable `next_batch` -> `since` chaining; current adapter keeps the token only in memory | Specification is active and publishes releases/security policy; no pinned homeserver/client SDK exists in Fabric | **Adapter blocked.** c018 must persist the cursor, pin/test a homeserver target, and prove replay, E2EE decision and echo suppression |
| ATProto Jetstream | raw WebSocket adapter; no client package pin | Current Jetstream v2 cursors are inclusive, instance-local and retained for 36h by default; durable consumers save the last processed cursor. Public legacy v1 uses timestamp cursors | Repository is active (`v0.2.2` observed); current Fabric reconnects without persisting or supplying a cursor and does not declare v1/v2 | **Adapter blocked.** c019 must choose v1/v2, persist the matching cursor, handle expiry/resnapshot and prove restart/echo behavior |
| UniFFI | `uniffi 0.31.2`, checksum `46eefd54…70267` | Full upstream support is Kotlin, Swift and Python. Bindgen and runtime versions must match; 0.31 changed command/checksum compatibility | 0.31.2 fixed Kotlin/Swift async and ABI issues; latest observed tag is 0.32.1 | **Retain only for selected platforms.** c022 pins generator/runtime together and proves produced packages; no Dart/Go/C# support is inferred |
| wasm-bindgen | `0.2.125`, checksum `8ddb3f79…fca1a` | Required by the existing browser/WASM surface; exact output must match the selected toolchain | No direct finding in the current RustSec result; transitive browser dependencies still follow the audit gate | **Retain.** c022 proves generated browser artifacts and cancellation |

## Current Rust dependency security gate

Command: `cargo audit --json` from Fabric HEAD `52ab9b3874abbe35bbca73720d85578f3db70943`.
Database commit: `e2e640471715167f73e22eaf761f2e547adafeec`, updated
2026-09-14. Lockfile dependency count: 1,070.

The scan reports 8 vulnerabilities, 7 unmaintained warnings, 4 unsound warnings,
and 2 yanked packages. Directly relevant release blockers include:

- `quinn-proto 0.11.14` through the pinned Iggy fork (remote memory exhaustion);
- `rustls 0.23.40` throughout Iggy/HTTP/Surreal paths (TLS encryption-level flaw);
- `h2 0.4.15` on gateway HTTP/tonic paths (unbounded empty DATA frames);
- `ammonia 4.1.2` and other affected packages through SurrealDB 3.1.5;
- `rsa 0.9.10` through LiveKit/Surreal JWT support (no patched release); and
- `im`/`sized-chunks` through Loro (unmaintained and safe-code unsoundness).

Therefore c003 does not qualify the current full lockfile for production. The
revised proposal is to keep the selected architectural adapters while assigning
version/closure remediation and exact compatibility proofs to c004, c005, c007,
c008, c013, c017 and c020. c016/c023 must run a current source-bound audit and
accept no applicable vulnerability, unsoundness, yanked package, or unexplained
unmaintained dependency.

## Cross-project audit boundary

The same current advisory database was applied to Gate and Forge; production
pnpm graphs were checked for Fabric, PEM and ASO. The compact source-bound result
is `integration-audit-summary.json`:

- Gate: 10 Rust vulnerabilities, 1 unmaintained, 4 unsound, 1 yanked;
- Forge: 6 Rust vulnerabilities, 1 unsound, 3 yanked, including Wasmtime 46.0.1;
- PEM monorepo: 2 critical, 13 high, 12 moderate and 5 low npm advisories;
- ASO web: 7 high and 6 moderate npm advisories, all reported through its
  resolved React Router line; and
- Fabric production npm graph: zero reported, while Fabric Rust remains blocked.

These results revise the integration proposal as well: c004/c005 own deployment
and Gate runtime closure, c011 owns Forge adapter closure, c012 owns the exact
PEM client package graph, c014/c015 own the ASO browser graph, and c023 requires
current clean or explicitly non-applicable receipts from every repository. A
package reported only through a tool/dev application may be proven outside the
shipped artifact, but the count itself is never waived.

## Research sources

- Ory Keto health/API docs: <https://github.com/ory/keto/blob/master/internal/httpclient/README.md>
- Ory releases/support posture: <https://github.com/ory/keto/releases>
- Cedar API/changelog/security: <https://github.com/cedar-policy/cedar>
- pg-walstream source/current line: <https://github.com/isdaniel/pg-walstream>
- Apache Iggy source/releases: <https://github.com/apache/iggy>
- SurrealDB security policy/advisories: <https://github.com/surrealdb/surrealdb/security>
- redb 4.1 API: <https://docs.rs/redb/4.1.0/redb/>
- LiveKit Rust SDK releases/API: <https://github.com/livekit/rust-sdks/releases>
- str0m run-loop/API: <https://github.com/algesten/str0m>
- Matrix specification: <https://github.com/matrix-org/matrix-spec>
- Jetstream recovery contract: <https://github.com/bluesky-social/jetstream/blob/main/docs/README.md>
- UniFFI changelog: <https://github.com/mozilla/uniffi-rs/blob/main/CHANGELOG.md>
- Connect-ES compatibility: <https://github.com/connectrpc/connect-es>
- PGlite persistence/upgrades: <https://github.com/electric-sql/pglite/blob/main/docs/docs/upgrade.md>
- Loro update encoding: <https://github.com/loro-dev/loro>
- RustSec database receipt: local `cargo audit` output against the database commit above
