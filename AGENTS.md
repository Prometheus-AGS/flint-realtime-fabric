# AGENTS.md

Guidance for coding agents working in this repository.

The full project instructions live in **[CLAUDE.md](CLAUDE.md)** — architecture,
the absolute dependency rule, one-port-per-adapter, file-size limits, protobuf
contract rules, and the phase-gate protocol. Read it before generating code.

This file exists so agents that look for `AGENTS.md` by convention find the same
rules, and to surface the one policy that is most often violated by accident:

---

## Testing Policy — Local Integration Only (NON-NEGOTIABLE)

**CI/CD is never used to run tests. Ever.** All testing is local
full-integration testing against a locally-composed stack.

Forbidden: dispatching a workflow to test something, pushing to trigger a test
run, re-running a CI job to observe behaviour, or treating a CI result as
evidence that a gate may flip. **A task whose only completion path is a CI run
is unsatisfiable** and must be rewritten around a local run.

Permitted: CI for build, lint, typecheck, formatting and packaging. Local
`cargo test`, `vitest`, and full local integration runs.

The authoritative statement, with the worked example that motivated it, is in
[CLAUDE.md](CLAUDE.md) → *Testing Policy — Local Integration Only*.
