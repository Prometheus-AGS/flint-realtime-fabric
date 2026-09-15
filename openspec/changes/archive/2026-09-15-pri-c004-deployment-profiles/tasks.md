# Tasks — pri-c004-deployment-profiles

All items are planned. Do not check them off from a compile-only or historical receipt.

- [x] 1. Inspect the affected paths listed in design.md, record their source/dirty-diff baseline and verify dependency receipts: pri-c001-build-policy, pri-c002-local-fixtures, pri-c003-scope-research-contract.
- [x] 2. Complete the bounded change: Render supported full and restricted-shape profiles from pinned artifacts and explicit identity/media/secret inputs. Remove absolute developer paths and false-success health checks; reject endpoints outside profile authority.
- [x] 3. Establish and record acceptance A: Clean local installation of each selected profile reaches semantic readiness with real dependency handshakes; missing issuer/keys/broker/auth authority fails visibly.
- [x] 4. Establish and record acceptance B: Pinned images, declared secret sources, exposed ports and feature flags match the profile; release artifacts contain no dev authorization bypass. Each profile declares TLS termination, certificate/key/trust sources and external/internal network boundaries; valid HTTPS succeeds, invalid or missing TLS inputs fail startup/readiness, and prohibited plaintext or backend-bypass access is denied.
- [x] 5. Save a source-bound receipt under `.kbd-orchestrator/phases/production-readiness-integration/evidence/pri-c004-deployment-profiles/`; run applicable local quality checks and `openspec validate pri-c004-deployment-profiles --strict --no-interactive`. Resolve required review findings before archive.

## Execution notes

Runtime proof must name and run the exact local command against owned fixtures.
Public type/port edits require semver impact; touched files remain at most 500 lines.
Changes crossing repositories require each repository revision and applicable checks.
Unanswered operator decisions block dependent product work, not c003 research.
No CI test invocation, production deployment or KBD migration is part of these tasks.
