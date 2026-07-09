# release-signoff Specification

## Purpose
TBD - created by archiving change p17-c010. Update Purpose after archive.
## Requirements
### Requirement: The phase MUST close with a verified release sign-off

Phase-17 MUST end with a re-run of the full gate suite on the current tree, a security
spot-check confirming the phase-16 security fixes hold, and a release sign-off note
recording zero CRITICAL findings with HIGH triaged. Deferred planes MUST be re-affirmed
honestly, not presented as shipped.

#### Scenario: gates pass and sign-off records the result

- **WHEN** the phase-close verification runs the fmt/clippy/check/test gates
- **THEN** all gates pass and a sign-off note records zero CRITICAL, HIGH triaged, and the
  deferred planes with their rationale

#### Scenario: security fixes still hold

- **WHEN** the spot-check re-verifies c001/c002
- **THEN** no committed secret exists, the override is untracked, and JWT_ISSUER is
  mandatory in production

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

### Requirement: Phase-21 MUST close with an honest sign-off and keep the sovereign gate off until media flows end-to-end

Phase-21 MUST end with the release gate suite re-run green, `docs/SECURITY.md` §6 updated to
the str0m media plane's true status (1-to-1 RTP forwarding wired + layer-proven; two-peer
DTLS-connected proven; end-to-end media + N-peer/PLI deferred), a `CHANGELOG` Phase 21
section, and a sign-off note. `SFU_MODE=sovereign` MUST remain gated off because the
end-to-end (two connected peers exchange decoded media) proof is browser-gated and not yet
done. The deferred work MUST be seeded into a dedicated phase-22, not presented as shipped.

#### Scenario: gates pass and docs match reality

- **WHEN** the phase-close verification runs fmt/clippy/check/test
- **THEN** all pass and §6 records 1-to-1 forwarding proven with end-to-end media deferred

#### Scenario: the sovereign gate stays off until media flows end-to-end

- **WHEN** phase-21 closes with only layer/integration proofs (no browser end-to-end proof)
- **THEN** `SFU_MODE=sovereign` stays gated off and the flip is seeded to phase-22

### Requirement: Phase-22 MUST close with an honest sign-off and keep the sovereign gate off until media flows end-to-end

Phase-22 MUST end with the release gate suite re-run green, `docs/SECURITY.md` §6 updated to
the str0m media plane's true status (N-peer fan-out + PLI forwarding + gateway composition
present + layer-proven; end-to-end browser proof deferred), a `CHANGELOG` Phase 22 section, and
a sign-off note. `SFU_MODE=sovereign` MUST remain gated off because the end-to-end (two real
peers exchange decoded media) proof is browser-gated and not yet done. The deferred work MUST be
seeded into a dedicated phase-23, not presented as shipped.

#### Scenario: gates pass and docs match reality

- **WHEN** the phase-close verification runs fmt/clippy/check/test
- **THEN** all pass and §6 records the composed media plane with the E2E proof deferred

#### Scenario: the sovereign gate stays off until end-to-end media

- **WHEN** phase-22 closes with only layer/integration proofs (no browser E2E proof)
- **THEN** `SFU_MODE=sovereign` stays gated off and the flip is seeded to phase-23

