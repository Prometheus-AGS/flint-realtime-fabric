# release-signoff (delta)

## ADDED Requirements

### Requirement: Phase-19 MUST close with an honest sign-off and a seeded successor phase

Phase-19 MUST end with the release gate suite re-run green, `docs/SECURITY.md` §6 updated to
each plane's true status (what shipped vs. re-affirmed deferred), a `CHANGELOG` Phase 19
section, and a sign-off note. The deferred XL items (full str0m SFU media loop; live
cross-node proofs) MUST be re-affirmed with rationale and seeded into a dedicated successor
phase, not presented as shipped.

#### Scenario: gates pass and docs match reality

- **WHEN** the phase-close verification runs fmt/clippy/check/test
- **THEN** all pass and §6 records what functions end-to-end vs. what is deferred

#### Scenario: the successor phase is seeded from proven findings

- **WHEN** phase-19 closes
- **THEN** a phase-20 seed captures the remaining sovereign SFU media loop (built on the
  c006 transport-loop proof) and the live cross-node proofs, with `SFU_MODE=sovereign`
  still gated off
