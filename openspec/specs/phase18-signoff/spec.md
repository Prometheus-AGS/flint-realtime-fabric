# phase18-signoff Specification

## Purpose
TBD - created by archiving change p18-c010. Update Purpose after archive.
## Requirements
### Requirement: Phase-18 MUST close with an honest re-audit sign-off

Phase-18 MUST end with the full gate suite re-run green, docs updated for what shipped vs.
still deferred (SECURITY.md §6, API-REFERENCE, CHANGELOG, README), and a sign-off note.
Deferred items MUST be re-affirmed with rationale, not presented as shipped.

#### Scenario: gates pass and docs match reality

- **WHEN** the phase-close verification runs fmt/clippy/check/test
- **THEN** all pass and the docs record what functions end-to-end vs. what is deferred

#### Scenario: deferred planes are re-affirmed, not hidden

- **WHEN** a plane is still incomplete (full SFU, LiveKit inbound, full OIDC, ATProto
  gateway wiring, Dart async transport)
- **THEN** it is documented as deferred with rationale and remains gated off

