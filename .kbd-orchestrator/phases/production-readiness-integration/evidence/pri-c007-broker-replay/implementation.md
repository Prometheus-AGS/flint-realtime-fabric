# c007 broker replay implementation

Captured: `2026-09-15T16:14:00Z`

Status: **IMPLEMENTED; LOCAL ACCEPTANCE PASS**

## Position and checkpoint mapping

- Published envelopes use their stable `EventEnvelope.id` as Iggy's message ID,
  so a producer retry or process restart retains one broker identity.
- Because the pinned send acknowledgement has no append position, `publish`
  snapshots the partition around the send and scans the bounded new interval by
  stable message ID. A deduplicated retry falls back to retained-history lookup.
  It returns the exact authoritative Iggy position, including under concurrent
  publishers. Every delivered envelope carries that same position.
- Explicit offsets are inclusive. A caller that stored the last successfully
  processed position resumes with `last.next()`. `Offset::BEGINNING` maps to
  Iggy's `First` strategy.
- Polling explicitly disables Iggy auto-commit. Only `ack` or `seek` writes the
  durable named consumer checkpoint.

## Replay lifecycle

- Each subscription owns a dedicated authenticated Iggy connection. Receiver
  drop wins a cancellation select, stops the long poll and disconnects that
  client. This prevents a cancelled long-poll response from corrupting a later
  command on the shared publish/admin connection.
- Topics converge to a 24-hour expiry while preserving existing compression,
  replication and size settings. A non-beginning cursor below the retained
  partition floor fails with the stable `resnapshot_required:` marker.
- The local fixture enables one-second durable message saving with fsync,
  partition fsync after every message, state fsync and persisted message-ID
  deduplication. It runs the exact SDK-compatible server image
  `iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5`.

## Compatibility

The `LogBroker` method signatures are unchanged. Their public documentation now
defines authoritative publish/subscriber offsets, the inclusive cursor
convention and typed resnapshot marker. This corrects previously false runtime
behavior within the current pre-1.0 API; no new source signature is introduced.
The accepted c006 v2 watch contract remains unchanged.

All touched source and fixture files are below 500 lines. Acceptance A and B
are recorded separately only after the matching owned runtime scenarios pass.
