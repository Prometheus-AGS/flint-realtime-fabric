# Tasks — p38-c004-gateway-main-size-and-enforcement

> **DONE** = main.rs is under the cap and a check exists.
> **PROVEN** = the check was observed to FAIL on an oversized file and PASS after.

- [x] T1: Confirm `main.rs` is still over 500 lines (`wc -l`).
- [x] T2: Extract `ensure_entities_channel` into its own module; keep behaviour identical
      (the fixture failure must stay fatal, not revert to a warn!).
- [x] T3: Confirm `cargo check -p frf-gateway`, `cargo clippy … -D warnings -W clippy::pedantic`
      and `cargo fmt --check` all still pass.
      **RESULT (2026-09-14):** `cargo check -p frf-gateway` exit 0; `clippy --lib --bins
      -D warnings -W clippy::pedantic` exit 0; `cargo fmt --check --all` PASS (after one
      rustfmt pass fixing my own import-group reflow). `main.rs` 504 -> 425 lines.
      Behaviour-identical evidence: the moved function bodies diff IDENTICAL against the
      pre-edit backup — only visibility (`pub(crate)`) and doc comments differ.
      **One gate fails and it is NOT this change's:** `clippy --features dev-endpoints`
      exits 101 with `E0063: missing field 'subject'` at `routes/dev.rs:161`. Proven
      pre-existing by stashing this work and re-running against unmodified HEAD (same
      error, same exit). See the proposal's second finding.
- [x] T4: Add a file-size check covering all languages, capped at 500 lines.
- [x] T5: Sabotage the check — point it at a deliberately oversized file and confirm it
      FAILS, then restore. A check that has never failed is a hypothesis.
      **RESULT (2026-09-14): the guard bites, and sabotage found a real flaw in it.**

      1. *It bites.* A 601-line file produced `FAIL: 1 file(s) over the 500-line cap`,
         exit 1; exit 0 again after removal. Not vacuous.
      2. *It had a blind spot — mine.* The first probe exposed that `git ls-files` lists
         **tracked files only**, so a brand-new oversized file passed locally until staged.
         Harmless in CI (checkout yields a fully tracked tree) but wrong on a developer's
         machine, where the newest file is the likeliest to be over. Fixed by unioning
         `git ls-files --others --exclude-standard`, then re-proved with an **unstaged**
         601-line probe: exit 1. Found by testing the script, not by reading it.
      3. *A confounded probe nearly became a false finding.* An intermediate check reported
         the `.gitignore`/`target/` exclusion as broken. It was not: the previous probe was
         still on disk and the test branched on a bare exit code with output suppressed.
         Re-run with a verified exit-0 baseline and only the ignored file present, the
         result is PASS/exit 0 — both defenses (path filter and `--exclude-standard`) work.

      Also verified: `envelope_pb.ts` (489, generated) stays excluded; `bash -n` and
      `shellcheck` both clean on the edited script.
- [x] T6: Record the four near-cap files so the next overage is anticipated.
      **DONE — and not as a hand-maintained list.** `scripts/check-file-size.sh` emits the
      near-cap set on every run (WARN_WITHIN=25), so the record cannot drift out of date
      the way a static list in a document would. Current output:

      ```
      Within 25 lines of the 500-line cap (not a failure):
        498  crates/frf-gateway/src/config/mod.rs
        495  crates/frf-sdk-rust/src/shape.rs
        484  crates/frf-app/src/shape/tests.rs
        477  crates/frf-app/src/shape/lease.rs
      PASS: no file over 500 lines (271 checked, 4 near the cap).
      ```

      Note `sdks/ts/src/gen/flint/v1/envelope_pb.ts` (489) is correctly ABSENT — the
      generated-code exclusion is verified behaviour, not stated intent.
