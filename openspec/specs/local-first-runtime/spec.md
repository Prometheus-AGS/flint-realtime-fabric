# local-first-runtime Specification

## Purpose
TBD - created by archiving change p37-c001-pem-runtime-scoped-state. Update Purpose after archive.

## Requirements

### Requirement: Runtime state SHALL be owned per runtime instance

Each local-first graph runtime SHALL own its pending-action set and its
sync-status store. No runtime's state may be observable or mutable by another
runtime in the same process.

#### Scenario: A second runtime hydrating does not erase the first's pending actions

- **GIVEN** runtime A holds an un-settled pending action
- **AND** runtime B is started against a different storage key
- **WHEN** runtime B hydrates a snapshot carrying its own pending actions
- **THEN** runtime A's pending action remains present
- **AND** runtime B does not observe runtime A's pending action

#### Scenario: Each runtime reports its own status

- **GIVEN** two runtimes started with different storage keys
- **WHEN** each is asked for its status
- **THEN** each reports its own storage key, not whichever runtime wrote last

#### Scenario: Standalone persistence helpers work without a runtime

- **GIVEN** no runtime has been started
- **WHEN** `persistGraphToStorage` or `hydrateGraphFromStorage` is called directly
- **THEN** the call succeeds against a process-wide fallback scope

### Requirement: Persisted storage SHALL NOT be namespaced by practice alone

The persisted namespace for a session's graph SHALL incorporate the principal's
identity and the replica generation in addition to the practice. Keying by
practice alone lets two principals in one practice share private data.

#### Scenario: Two identities in one practice do not share a namespace

- **GIVEN** two sessions with the same practice and different identities
- **WHEN** a storage key is computed for each
- **THEN** the two keys differ

#### Scenario: A user and an agent acting for them do not share a namespace

- **GIVEN** two sessions identical except that one principal is `user` and the other `agent`
- **WHEN** a storage key is computed for each
- **THEN** the two keys differ

#### Scenario: A replica generation invalidates the namespace

- **GIVEN** a session
- **WHEN** a storage key is computed
- **THEN** the key carries a replica-generation component that changes the namespace when the local schema changes

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
