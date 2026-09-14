# p38-c002 — Prove (or disprove) the issue #2 fix by sabotage

## Summary

Commit `788637a` fixed the real cause of issue #2 — channel ids minted randomly at publish
time — but its end-to-end verification was blocked by the handshake failure in c001. The
guard it added has **never been observed to fail**, which by this project's own standard
makes it a hypothesis rather than a guard.

## Evidence

`crates/frf-broker-iggy/tests/publish_subscribe.rs:85` —
`a_subscriber_knowing_only_the_well_known_id_receives_published_events` publishes and
subscribes without threading a channel id between the two halves, so it should fail when a
publisher reverts to `ChannelId::new()`. That property is untested: the suite has never run
against a live server.

Three vacuous guards were found and removed in this codebase on 2026-09-14 alone. A test
that passes when its mechanism is deleted proves nothing.

## Why this is independent

It is not — this change has a hard dependency on c001. Without a server that completes a
handshake, neither the pass nor the sabotage can be observed.

## Scope

Run the ignored suite against the working server from c001. Then revert
`ChannelId::WELL_KNOWN_ENTITIES` to `ChannelId::new()` and confirm the test **fails**.
Record both outcomes. Close issue #2 only on demonstrated receipt — not on a green compile.

## Non-goals

- Changing any production code. If the guard does not bite, that is a finding to report,
  not a licence to adjust the test until it passes.

## Files

`crates/frf-broker-iggy/tests/publish_subscribe.rs` (temporarily, for sabotage only —
reverted), plus the recorded run output.
