# release-signoff (delta)

## ADDED Requirements

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
