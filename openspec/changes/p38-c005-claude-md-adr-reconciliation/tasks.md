# Tasks — p38-c005-claude-md-adr-reconciliation

> **DONE** = every claim in CLAUDE.md matches an ADR or the code.
> **PROVEN** = each corrected row was checked against the cited source, not assumed.

- [ ] T1: Re-verify each drift row against its ADR and the code before editing — do not
      trust this proposal's table without re-checking.
- [ ] T2: Correct the "Open Decisions" table: remove settled rows with an ADR pointer;
      keep genuinely open ones.
- [ ] T3: Correct `:102` (CRDT), `:256` (Dart/FRB), and the workspace tree (five crates).
- [ ] T4: Flag `dagger/codegen.ts` stage 4 as broken without fixing it.
- [ ] T5: Grep for the same stale claims elsewhere (AGENTS.md, README.md, website/docs)
      so the correction is not partial.
