# release-signoff (delta)

## ADDED Requirements

### Requirement: Phase-20 MUST close with an honest sign-off and a seeded successor phase

Phase-20 MUST end with the release gate suite re-run green, `docs/SECURITY.md` §6 updated to
the str0m media plane's true status (async engine + trickle ICE + DTLS-connected milestone
reached; media forwarding still deferred), a `CHANGELOG` Phase 20 section, and a sign-off
note. The deferred RTP forwarding media loop and the live G5 proofs MUST be re-affirmed with
rationale and seeded into a dedicated phase-21, not presented as shipped. `SFU_MODE=sovereign`
MUST remain gated off (connected ≠ media flowing).

#### Scenario: gates pass and docs match reality

- **WHEN** the phase-close verification runs fmt/clippy/check/test
- **THEN** all pass and §6 records the DTLS-connected milestone with media still deferred

#### Scenario: the successor phase is seeded for RTP forwarding

- **WHEN** phase-20 closes
- **THEN** a phase-21 seed captures RTP forwarding (built on the connected-milestone engine),
  the offerer/peer role, and the carried G5 live proofs — with `SFU_MODE=sovereign` still off
