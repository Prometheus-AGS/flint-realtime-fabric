# release-signoff (delta)

## ADDED Requirements

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
