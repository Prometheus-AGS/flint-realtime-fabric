# Tasks — p38-c002-prove-channel-id-fix

> **DONE** = both runs executed and their outputs recorded.
> **PROVEN** = the guard was observed to PASS clean and FAIL under sabotage.

- [x] T1: Confirm c001 landed and a server now accepts a client. If not, this change is
      BLOCKED — do not proceed.
- [x] T2: Run `IGGY_TEST_CONNECTION_STRING=… cargo test -p frf-broker-iggy -- --ignored`
      and record the result verbatim.
- [x] T3: Sabotage — revert `ChannelId::WELL_KNOWN_ENTITIES` to `ChannelId::new()` in
      `crates/frf-postgres-cdc/src/consumer.rs`, re-run, and confirm
      `a_subscriber_knowing_only_the_well_known_id_receives_published_events` FAILS.

      **THIS SABOTAGE CANNOT BITE — the task text is wrong, and I wrote it.**
      `crates/frf-broker-iggy/Cargo.toml` has no `frf-postgres-cdc` dependency, so
      `consumer.rs:109` is **unreachable** from this test binary. The guard builds its own
      `Channel { id: ChannelId::WELL_KNOWN_ENTITIES, .. }` inline and subscribes with the
      constant directly; nothing it executes passes through the CDC consumer. Editing
      `consumer.rs` would leave the test green and produce a recorded "sabotage confirmed"
      that proved nothing — the same vacuity this change exists to eliminate, moved one
      level up. **A sabotage that cannot fail the guard is as empty as a guard that cannot
      fail.**

      Two other candidates also fail to bite, for the record:
      - Changing the constant in `ids.rs:62`: both halves read it, so they still meet.
      - Changing only the subscriber's id: publish would then create that stream too
        (publish is self-healing since `788637a`), so they could still meet.

      **The sabotage that genuinely severs the tested property** is breaking the
      *publisher's* inline id while the subscriber keeps the constant — that is exactly
      the independence the guard asserts, and exactly how the CDC consumer shipped broken.
      Run as: `id: ChannelId::WELL_KNOWN_ENTITIES` → `id: ChannelId::new()` in the
      publisher half of the test, re-run, expect FAIL, then restore.

      T3 is executed BOTH ways below: as written (to document that it is inert) and
      correctly (to actually prove the guard).

      **T3a — AS WRITTEN: CONFIRMED INERT (executed, not argued).** With
      `consumer.rs:109` set to `ChannelId::new()`, the guard still passed — `2 passed;
      0 failed`, exit 0. The sabotage named in the original task text cannot fail this
      test. `consumer.rs` restored; `git diff` empty.

      **T3b — AIMED CORRECTLY: THE GUARD BIT.** Publisher id → `ChannelId::new()`,
      subscriber left on the constant. Result: **exit 101, test FAILED**:

      ```
      subscribe failed: Transport("Stream with name:
        channel-00000000-0000-0000-0000-000000000001 was not found.")
      ```

      That message is **verbatim the error reported in issue #2**. The sabotage reproduced
      the exact original symptom, which is the strongest available evidence that this guard
      detects the defect it claims to. Restored; `git diff` on the test empty.

      **A false negative was avoided by preparation, not luck.** Before T3b the well-known
      stream already held **4 messages** from earlier runs (plus three orphaned random-id
      streams from the pre-fix era), and the payload literal is identical across runs. The
      subscriber would very likely have read a stale message and PASSED under sabotage —
      recording "the guard does not bite" about a guard that does. The volume was wiped
      first (`docker compose down -v`; server then logged `Loaded 0 stream(s)`).
- [x] T4: Restore the constant; re-run; confirm green again.
- [x] T5: Comment on issue #2 with both outcomes linked, and close it only if T2 and T3
      both behaved as required. If the guard did not bite, report that instead.
