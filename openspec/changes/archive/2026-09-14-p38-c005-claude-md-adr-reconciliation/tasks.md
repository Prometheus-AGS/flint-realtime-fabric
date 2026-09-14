# Tasks — p38-c005-claude-md-adr-reconciliation

> **DONE** = every claim in CLAUDE.md matches an ADR or the code.
> **PROVEN** = each corrected row was checked against the cited source, not assumed.

- [x] T1: Re-verify each drift row against its ADR and the code before editing — do not
      trust this proposal's table without re-checking.
- [x] T2: Correct the "Open Decisions" table: remove settled rows with an ADR pointer;
      keep genuinely open ones.
- [x] T3: Correct `:102` (CRDT), `:256` (Dart/FRB), and the workspace tree (five crates).
- [x] T4: Flag `dagger/codegen.ts` stage 4 as broken without fixing it.

> **FOREIGN UNCOMMITTED WORK IN THE TREE — deliberately not committed by this change.**
> Three source files are dirty that c005 never touched:
> `crates/frf-app/src/shape/mod.rs`, `crates/frf-app/src/shape/tests.rs`,
> `crates/frf-gateway/src/routes/shape.rs`. All three carry mtime `11:57:55`, hours before
> this change's doc edits (`13:28`) and before this `/kbd-apply` began, so they are not
> mine and not hook output.
>
> The diff is coherent, deliberate engineering: a new `ShapeUseCaseError::HandleExpired`
> variant, a `same_authority` comparison that distinguishes an expired lease from a genuine
> handle mismatch, a comment recording that Gate mints a fresh JWT per continuation, and a
> matching test adjustment. That is in-progress ADR-009 shape-lease work.
>
> **It is left untouched in the working tree and excluded from c005's commit.** A
> documentation-only change that swept in three source files of someone else's shape-lease
> work would misattribute authorship and bury real changes inside a docs commit. `git add`
> is scoped to exactly the files this change edited.
- [x] T5: Grep for the same stale claims elsewhere (AGENTS.md, README.md, website/docs)

> **DELIBERATELY NOT CHANGED — `website/docs/case-studies/knowme.md:70`.**
> That line reads "**Status: built.** `flutter_rust_bridge` is pinned at 2.12.0". It
> matched the FRB sweep, but reading the surrounding text shows it describes **KnowMe**, a
> different project ("Platform code (Flutter and Riverpod on mobile; Tauri 2, React 19 and
> Zustand on desktop)", "The most transferable idea in KnowMe"). FRB 2.12.0 is a true
> statement about KnowMe's codebase and has nothing to do with FRF's ADR-003 decision.
> Correcting it would have been misattribution driven by a grep match rather than by
> reading. Left exactly as it is; recorded so the next sweep does not "fix" it.
>
> **Scope actually corrected (current-state docs only):** `CLAUDE.md` (Open Decisions
> table, `:50` tree comment, `:102` CRDT, `:108` FFI, `:113` versions note, `:256` SDK
> strategy, plus five missing member crates in the tree), `README.md` (`:239`, `:256`),
> `docs/IMPLEMENTATION-PLAN.md` (`:77`, `:180`, `:316`, `:353`, `:356`), `dagger/README.md`
> (the `frb-dart` row).
>
> **Deliberately untouched as historical record:** everything under
> `.kbd-orchestrator/phases/` (assessments and reflections written when those decisions
> genuinely were open), `openspec/changes/archive/`, and `CHANGELOG.md:563` — which
> correctly records that a past drift was fixed. Rewriting those would falsify history.
      so the correction is not partial.
