# replica-materialization Specification

## Purpose
TBD - created by archiving change p37-c004-aso-replica-runtime. Update Purpose after archive.

## Requirements

### Requirement: The replica runtime SHALL be schema-agnostic

The component that materializes shape chunks into local tables SHALL take its
target table and columns as data supplied by the caller. It SHALL NOT reference
any application table or column by name, so that the privacy boundary declared
by the shape catalog cannot be widened by the runtime.

#### Scenario: Rows are projected onto the declared columns

- **GIVEN** a target declaring a set of columns
- **AND** a row carrying an additional undeclared field
- **WHEN** the chunk is written
- **THEN** only the declared columns are written
- **AND** the undeclared field does not reach local storage

#### Scenario: A malformed target is refused before any write

- **GIVEN** a target whose table or column name is not a valid identifier
- **WHEN** a write or truncate is attempted
- **THEN** it is refused and no statement reaches the database

#### Scenario: A target omitting its id column is refused

- **GIVEN** a target whose declared columns do not include its id column
- **WHEN** the target is validated
- **THEN** it is refused

### Requirement: A chunk SHALL land whole or not at all

Writing a chunk SHALL be atomic. A partially applied chunk leaves the caller
unable to say what its resume checkpoint describes.

#### Scenario: A failed row rolls the chunk back

- **GIVEN** a chunk whose write fails partway
- **WHEN** the failure occurs
- **THEN** the transaction is rolled back and not committed

### Requirement: One database revision SHALL publish as one graph revision

Materialized rows SHALL reach the entity graph in a single publication, so that
a subscriber observes either the previous complete projection or the next one,
never a partially updated relationship.

#### Scenario: Multiple tables publish in one update

- **GIVEN** a revision spanning three related entity types
- **WHEN** it is published
- **THEN** exactly one store update occurs
- **AND** the non-primary types are carried as side batches of that update

#### Scenario: A subscriber never observes a partial relationship

- **GIVEN** a subscriber recording what it sees on each publication
- **WHEN** a revision containing a citation and the document it references is published
- **THEN** every observation containing the citation also contains the document

#### Scenario: An empty revision publishes nothing

- **GIVEN** a revision whose batches contain no entries
- **WHEN** it is published
- **THEN** no store update occurs

### Requirement: An untrustworthy replica SHALL be rebuilt, not merged

When the server signals must-refetch, or a stored checkpoint does not describe
the stored rows, the replica generation SHALL be incremented and its rows
removed. A fresh snapshot SHALL NOT be merged into the existing rows.

#### Scenario: The generation is incremented before rows are cleared

- **GIVEN** a rebuild is triggered
- **WHEN** it runs
- **THEN** the generation is incremented before any row is deleted
- **AND** an interruption after the increment leaves every prior checkpoint recognisably stale

#### Scenario: Rebuild clears rather than reconciles

- **GIVEN** a rebuild across several targets
- **WHEN** it runs
- **THEN** each target's rows are deleted
- **AND** no row-level insert or update is issued as part of the rebuild

#### Scenario: Must-refetch takes precedence over a rejected resume

- **GIVEN** both a must-refetch signal and a rejected resume
- **WHEN** the trigger is determined
- **THEN** it is reported as must-refetch

### Requirement: Exactly one context SHALL own the replica for writing

Migration and write access to the local replica SHALL be held by one context at
a time under an expiring lease, so that two tabs cannot both migrate or write it.
Read access SHALL NOT require the lease.

#### Scenario: A second context is refused while the lease is live

- **GIVEN** a live lease held by another context
- **WHEN** a second context attempts to acquire it
- **THEN** acquisition is refused and the current holder is reported

#### Scenario: An expired lease may be taken over

- **GIVEN** a lease whose expiry has passed
- **WHEN** another context attempts to acquire it
- **THEN** acquisition is granted

#### Scenario: Two racing contexts do not both acquire

- **GIVEN** no current lease
- **WHEN** two contexts attempt to acquire simultaneously
- **THEN** exactly one is granted

#### Scenario: A displaced holder cannot renew

- **GIVEN** a context whose lease has been taken by another
- **WHEN** it attempts to renew
- **THEN** renewal fails and it no longer claims to hold the lease

### Requirement: Applied migrations and the replica generation SHALL be recorded

The runtime SHALL record which migrations have been applied and which generation
the replica is on, rather than inferring either from the schema's current shape.

#### Scenario: A migration is applied once

- **GIVEN** a migration already recorded as applied
- **WHEN** it is applied again
- **THEN** its SQL is not re-executed

#### Scenario: A rebuild advances the recorded generation

- **GIVEN** a replica on a generation
- **WHEN** a rebuild occurs
- **THEN** the recorded generation is greater than before
