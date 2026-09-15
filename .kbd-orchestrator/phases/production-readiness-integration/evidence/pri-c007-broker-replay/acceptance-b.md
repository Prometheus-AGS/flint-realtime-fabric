# c007 acceptance B — identity, isolation, expiry and cancellation

Captured: `2026-09-15T17:28:40Z`

Status: **PASS**

## Producer identity and broker position

The core live scenario first supplied source-local offsets `7001` and `7002`.
Delivered envelopes instead carried Iggy's partition positions `0` and `1`.
After creating a new producer client, an event that again supplied source-local
offset `7001` received the next broker position (`2`). Producer process state
therefore cannot reuse or manufacture a replay position.

Publishing an already-used stable event ID did not append a message. The same
deduplication check passed after a server restart, demonstrating that stable
identity rather than process-local state controls retries.

Two independent broker clients also published concurrently with the same
source-local offset. `publish` returned distinct broker positions, and the
subsequent deliveries matched each event ID to its exact returned position.

## Consumer and partition boundaries

Two named consumers independently read the first event. Polling either consumer
did not advance its durable checkpoint; only the explicitly acknowledged
consumer stored a position. Each channel maps to one Iggy stream and a one-
partition `events` topic. Runtime inspection required exactly partition ID `1`,
matching the adapter's stable single-partition mapping and preserving total
order within a channel. The current `LogBroker` contract does not advertise
cross-partition ordering.

## Retention and resnapshot

Every normal topic was runtime-inspected with a 24-hour expiry. To exercise the
boundary without waiting 24 hours, the owned fixture created a separate topic,
published two oversized messages into separate 1 KB segments, changed only that
topic to a one-second expiry, and waited for Iggy's enabled cleaner. Server logs
record `Deleted 2 segments and 2 messages`. Subscribing at the now-expired
explicit second position failed immediately with `PortError::NotFound` whose
message began `resnapshot_required:`. It did not silently begin at the oldest
available position.

The zero-message retained-floor calculation is also covered by a unit test. A
client-wide beginning request remains the explicit snapshot/replay-from-floor
sentinel; c010 owns opaque watch-checkpoint validation.

## Cancellation

The live scenario dropped multiple receivers while long polls were pending,
then successfully published and opened subsequent subscriptions. A focused unit
test makes receiver closure win against a pending future. Each subscription owns
and closes a dedicated Iggy connection, so a late long-poll response cannot be
misread by the shared publish/admin client.

## Actual fixture durability

The server's rendered startup configuration and runtime logs recorded:

| Setting | Actual value |
|---|---|
| message saver | enabled, 1 second, fsync enabled |
| partition write | fsync enabled, save threshold 1 message |
| state log | fsync enabled |
| message cleaner | enabled, 1 second |
| segment size | 1 KB for deterministic expiry exercise |
| message-ID deduplication | enabled, 10,000 entries, 24-hour expiry |
| normal topic retention | 86,400 seconds |
| partition count | exactly 1 |

The restart verification loaded the checkpoint and messages from the owned
volume before replay. Server log SHA-256:
`b2675441e9d9bf49c37edeff8b8ebb482b11981dafedd5c7d2b5ad4077935e48`.
Core log SHA-256:
`99ca0646436b121d79968050920ca0c45eb10cab7fe33134aa746306a21a1701`.

The local command exited 0 and removed its owned containers and volume. No CI
test run supplied evidence.
