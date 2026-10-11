# Refinement decisions — pri-c007-broker-replay

- Use Iggy's partition offset for both the `publish` return and delivered
  replay/checkpoint position. Resolve the stable message ID within the bounded
  post-send append interval, with retained-history fallback for deduplicated
  retries because the pinned send acknowledgement has no append position.
- Disable polling auto-commit and write consumer positions only through explicit
  `ack` or `seek` after application processing.
- Map stable domain event IDs to Iggy message IDs so retry and restart use the
  broker's persisted deduplicator.
- Give every long-poll subscription a dedicated connection and close it when
  the receiver disappears, avoiding shared-command response corruption.
- Keep one ordered partition per channel until the public port models partition
  metadata. Explicit offsets are inclusive; callers resume with `last.next()`.
- Configure normal topics for at least 24 hours and fail an expired explicit
  position with `resnapshot_required:`. Use a short-lived owned test topic and
  small segments to exercise physical cleanup without weakening normal topics.
- Pin the exact server build matching the Cargo fork revision and record actual
  fsync, saver, cleaner and deduplication settings in the runtime receipt.
