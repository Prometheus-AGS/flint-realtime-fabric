# Tasks — p38-c002-prove-channel-id-fix

> **DONE** = both runs executed and their outputs recorded.
> **PROVEN** = the guard was observed to PASS clean and FAIL under sabotage.

- [ ] T1: Confirm c001 landed and a server now accepts a client. If not, this change is
      BLOCKED — do not proceed.
- [ ] T2: Run `IGGY_TEST_CONNECTION_STRING=… cargo test -p frf-broker-iggy -- --ignored`
      and record the result verbatim.
- [ ] T3: Sabotage — revert `ChannelId::WELL_KNOWN_ENTITIES` to `ChannelId::new()` in
      `crates/frf-postgres-cdc/src/consumer.rs`, re-run, and confirm
      `a_subscriber_knowing_only_the_well_known_id_receives_published_events` FAILS.
- [ ] T4: Restore the constant; re-run; confirm green again.
- [ ] T5: Comment on issue #2 with both outcomes linked, and close it only if T2 and T3
      both behaved as required. If the guard did not bite, report that instead.
