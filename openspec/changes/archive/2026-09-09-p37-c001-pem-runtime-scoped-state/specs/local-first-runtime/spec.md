# local-first-runtime

## ADDED Requirements

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
