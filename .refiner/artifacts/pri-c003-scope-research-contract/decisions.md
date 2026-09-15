# Refinement decisions — pri-c003-scope-research-contract

- Accept the conservative Q1-Q7 defaults authorized in `execution.md`; keep Q8
  separate and leave legacy KBD authority unchanged.
- Qualify `full` and `shape-only` independently and retain all deferred phase
  goals behind their own gates.
- Treat current audit findings and mutable image tags as production blockers.
  Preserve adapter architecture while assigning compatible version/digest and
  behavior proof to each owning later change.
- Record Gate/ASO source drift as a warning and require recapture before their
  next implementation work; c003 remains a dated research receipt.
