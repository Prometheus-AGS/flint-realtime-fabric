# p38-c004 — Repair the file-size violation and add enforcement

## Summary

`crates/frf-gateway/src/main.rs` is 504 lines against a hard 500-line cap. Constraint **R5**
in `.kbd-orchestrator/constraints.md` marks that **BLOCKING** — it fails the project's own QA
gate. It is currently the only such violation in the workspace.

## Evidence

The file was 487 lines before commit `788637a` and is 504 after: **this session's own commit
caused it**, by extracting `ensure_entities_channel` into the same file rather than a new
module.

Four more files sit within 25 lines of the cap and will cross it next:

| File | Lines |
|---|---|
| `crates/frf-gateway/src/config/mod.rs` | 498 |
| `crates/frf-sdk-rust/src/shape.rs` | 495 |
| `crates/frf-app/src/shape/tests.rs` | 484 |
| `crates/frf-app/src/shape/lease.rs` | 477 |

The cap is stated in CLAUDE.md and in `constraints.md` and enforced by **nothing** — which is
why it was crossed silently rather than caught.

## Why this is independent

It touches `crates/frf-gateway/` only. No dependency on c001's outcome.

## Scope

Move `ensure_entities_channel` (and plausibly the telemetry init) out of `main.rs` into a
`bootstrap` module. Then add an enforcement check so the next overage fails visibly — a
script or `just` target, or a CI **lint** step. CI may lint; per AGENTS.md it may never run
tests, and a line-count check is a lint.

## Non-goals

- Splitting the four near-cap files. They are recorded here as the next to cross, not fixed.
- **Removing the `cargo test` job from CI.** See the finding below — it is real, but it is
  not this change's scope.

## Finding, out of scope — CI runs tests, which the policy forbids

Found while locating the wiring point for T4's lint.

`.github/workflows/ci.yml:50-61` defines:

```yaml
  test:
    name: Test suite
    runs-on: ubuntu-latest
    steps:
      ...
      - name: cargo test
        run: cargo test --all
```

`AGENTS.md:14-26` is unambiguous:

> **CI/CD is never used to run tests. Ever.** All testing is local full-integration testing
> against a locally-composed stack. … Permitted: CI for build, lint, typecheck, formatting
> and packaging.

CLAUDE.md carries the identical statement. So the repository's CI has been violating its own
non-negotiable policy.

`git log -- .github/workflows/ci.yml` traces this job to `6e549e8` ("complete Phase 0") —
it has been there since the first CI and **predates the policy**, rather than being
introduced in defiance of it. That makes it drift, not defiance, but it is drift in the one
place the project declared absolute.

**Why it is not fixed here.** This change adds a *lint* to CI, which the policy explicitly
permits. Deleting a test job inside a file-size change would be scope creep on a workflow
file this change otherwise only appends to, and the decision — delete the job, or amend the
policy to match reality — belongs to the operator, not to c004.

## Second finding, out of scope — CI is red today on the dev-endpoints config

`ci.yml:45` runs:

```
cargo clippy --workspace --lib --bins --features frf-gateway/dev-endpoints -- -D warnings -W clippy::pedantic
```

That command **fails**:

```
error[E0063]: missing field `subject` in initializer of `frf_domain::SignalEnvelope`
   --> crates/frf-gateway/src/routes/dev.rs:161:24
```

**Proven pre-existing, not caused by this change.** The working tree was stashed
(`git stash push --include-untracked crates/frf-gateway/`, with the new `bootstrap.rs`
moved aside) and the same command run against unmodified HEAD — it exits 101 with the
identical error. Work was restored intact afterwards.

**Cause.** `SignalEnvelope` gained a `subject` field in `e649db3` (phases 20–35) —
`crates/frf-domain/src/signal.rs:44`, "Authenticated identity (JWT subject) of the sender,
stamped server-side after token verification (p24-c003)… for the ADR-007 media `view`
check". The construction site at `routes/dev.rs:161` was never updated. It escapes the
default build because that code is behind the `dev-endpoints` feature gate, so
`cargo check --workspace` is green while the feature-gated config has been broken since
that commit.

**Why it is not fixed here.** It is a one-line fix, but it is a different defect in a
different file under a different feature gate, and c004 is a file-size change. Bundling it
would make the diff dishonest about what was verified. Recorded for the operator.

## Files

`crates/frf-gateway/src/main.rs`, a new `crates/frf-gateway/src/bootstrap.rs` (or module
dir), and the enforcement check.
