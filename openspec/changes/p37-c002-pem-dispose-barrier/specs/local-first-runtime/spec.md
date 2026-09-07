# local-first-runtime

## MODIFIED Requirements

### Requirement: Disposal SHALL drain in-flight persistence

`dispose()` SHALL return a promise that resolves only once every persistence
write started by the runtime has settled. Cancelling a scheduled write is not
sufficient: a write already in flight MUST be awaited, so that a caller which
awaits disposal is guaranteed no further write from that runtime lands.

#### Scenario: Disposal does not resolve while a write is in flight

- **GIVEN** a runtime whose storage write has started but not settled
- **WHEN** `dispose()` is called
- **THEN** the returned promise does not resolve
- **AND** once the write settles, the promise resolves

#### Scenario: A write started immediately before disposal is covered

- **GIVEN** a runtime
- **WHEN** a persist is started and `dispose()` is called with no await between them
- **THEN** the disposal promise resolves only after that write has settled

#### Scenario: Disposal is idempotent

- **GIVEN** a runtime
- **WHEN** `dispose()` is called twice
- **THEN** both calls return the same promise
- **AND** teardown happens once

#### Scenario: No write is scheduled after disposal

- **GIVEN** a disposed runtime
- **WHEN** its graph store is mutated
- **THEN** no further persistence write reaches storage

### Requirement: A session owner SHALL sequence disposal against the next open

Because a React effect cleanup is synchronous and cannot await a promise, the
disposal barrier SHALL be held by a session owner rather than by the cleanup.
The owner SHALL ensure at most one live runtime, and SHALL await an outgoing
runtime's drain before opening a replacement.

#### Scenario: Reopening the same key reuses the live session

- **GIVEN** a live session for a storage key
- **WHEN** the same key is opened again
- **THEN** the existing session is returned rather than a second one created

#### Scenario: Concurrent opens coalesce

- **GIVEN** no live session
- **WHEN** two opens for the same key are issued before either resolves
- **THEN** both receive the same session

#### Scenario: Switching keys drains the outgoing session first

- **GIVEN** a live session for one key
- **WHEN** a different key is opened
- **THEN** the outgoing session is disposed and drained before the replacement opens

#### Scenario: Mount, unmount and remount yield one live session

- **GIVEN** a live session
- **WHEN** close is started without being awaited and the same key is opened again
- **THEN** exactly one session is live afterwards
- **AND** the outgoing session was disposed exactly once

#### Scenario: A failed teardown does not wedge the owner

- **GIVEN** a session whose disposal rejects
- **WHEN** close is called and a different key is then opened
- **THEN** close settles and the replacement session opens
