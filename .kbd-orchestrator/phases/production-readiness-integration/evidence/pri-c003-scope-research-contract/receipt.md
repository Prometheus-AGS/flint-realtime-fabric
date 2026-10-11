# Verification receipt — pri-c003-scope-research-contract

Captured: 2026-09-15T09:10:16Z

## Result

Implementation: COMPLETE for this research/decision slice.
Focused review: PASS, 0 critical, 1 warning, 0 suggestions.
Production readiness: BLOCKED by the later implementation and dependency gates
identified in the accepted contract; c003 does not certify a deployable release.

## Source binding

| Repository | Revision | status SHA-256 at final check |
|---|---|---|
| Fabric | `52ab9b3874abbe35bbca73720d85578f3db70943` | `3b07fedb5154989daff5247d54f2d651566a19374a2c364c9cef2718ced26cb3` |
| Gate | `97d65437c937a285fb000f5e42fddc8af4ff9934` | `1a2518bb485510fb39123dcf29179517e1f93629ae995eeaf4f919aac38c8046` |
| Forge | `dc313be3a044c65b05d845a9c34350bf5ca3ca3e` | clean, `e3b0c442…b855` |
| PEM | `071b9e5b06c31f6c7d9d191bdaa4a2e188d1d565` | clean, `e3b0c442…b855` |
| ASO | `d95242542060d1e19a6efe9143f57c015d6162f4` | `6881e7592666d1f7fd15745886563b6c253b21619d6a6d8bdd6516086297c2bf` |

Gate and ASO changed after the task-1 baseline. c003 made no edits there. This
receipt records the drift and repeats the independent review warning: later
changes must recapture and isolate those worktrees before using their source as
implementation evidence.

Lock hashes used for dependency identities:

- Fabric `Cargo.lock`: `605fe98e98ea824f057c9f38190977806be442a65a0a709c2007f41e00430736`
- Fabric `pnpm-lock.yaml`: `caf66ce216305b89e9db52bcc1251b17107fb8c03875e224245c22a3ff7f74ad`

## Artifact hashes

| Artifact | SHA-256 |
|---|---|
| `analysis.md` | `9814b8cd8ec94a062486f8001af1ff9d4c63e52514b7356bb1d6eef081d8a638` |
| `decision-log.md` | `bb0abec403c6eb5e203c38d46baeee768d1d0ee11901d29a806baaf5fae29c36` |
| `library-candidates.json` | `d31d4801a4ee0e0171147d0eab31afd7e10fe437b279d68a0796b521a2a70e5d` |
| `release-contract.md` | `29752a3109a4602a1edcf14e72be70b6f5b43270ba7ea165b3de9ab148c857bd` |
| `source-inventory.md` | `4142bf11c253a3abe7f8ae30e31107d8b21522eb6cc828f7787005bc3bb75989` |
| `dependency-qualification.md` | `ba099c68c1a66aaeed433c508abc229b108276d15f0e68928d5761e0bd001d9b` |
| `cargo-audit-summary.json` | `7ea984cd43ca2c5cae4b1aa3ad5afbb66ca0c972d487c50057f7b6d2071c8c3b` |
| `integration-audit-summary.json` | `aa0b7c0e0978945806eb258f18d3854eb117cf60d2047282be6fcd79f9a4f990` |
| focused review | `2fb2939ac3530df8afbb5668d46911bcf245c4f52cae79a4a4f2c4a161f1ab38` |

The receipt's own hash is intentionally omitted because adding it would be
self-referential. OpenSpec task/spec hashes before archive are retained in the
command log of this turn; the archive command produces the final path.

## Research and security commands

| Command / source | Result |
|---|---|
| `cargo metadata --locked --format-version 1` | PASS; resolved exact Fabric crate identities |
| Context7 queries for Keto, Cedar, SurrealDB, redb, LiveKit, str0m, Matrix, Jetstream, UniFFI, Connect-ES, PGlite and Loro | PASS; API/compatibility boundaries recorded |
| official GitHub repository/tag/release inspection | PASS; observed maintenance identities and mutable/deferred boundaries recorded |
| Fabric `cargo audit --json` | BLOCKED as expected: 8 vulnerabilities, 7 unmaintained, 4 unsound, 2 yanked |
| Gate `cargo audit --json` | BLOCKED as expected: 10 vulnerabilities, 1 unmaintained, 4 unsound, 1 yanked |
| Forge `cargo audit --json` | BLOCKED as expected: 6 vulnerabilities, 1 unsound, 3 yanked |
| Fabric `pnpm audit --prod --json` | zero production advisories reported |
| PEM `pnpm audit --prod --json` | BLOCKED: 2 critical, 13 high, 12 moderate, 5 low |
| ASO `pnpm --dir web audit --prod --json` | BLOCKED: 7 high, 6 moderate |

Blocked audit outcomes are evidence that revised dependency proposals are
necessary; they are not recorded as passing release gates.

## Local quality checks

| Command | Result |
|---|---|
| `openspec validate pri-c003-scope-research-contract --strict --no-interactive` | PASS: change is valid |
| `jq empty` over c003 JSON and `library-candidates.json` | PASS |
| `git diff --check` over phase/OpenSpec paths | PASS |
| per-file <= 500-line check over c003 evidence/review | PASS |
| acceptance-A required-term check against `release-contract.md` | PASS |
| fresh-context gpt-5.5 focused review | PASS: 0 critical, 1 warning |

No product code changed in c003, so a runtime test would not validate this
decision slice. The dependency scans and locked-metadata check are the relevant
local executable evidence. All product/runtime claims remain assigned to their
later local integration gates.
