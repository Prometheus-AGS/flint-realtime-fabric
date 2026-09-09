# replica-persistence Specification

## Purpose
TBD - created by archiving change p37-c003-pem-checkpoint-atomicity. Update Purpose after archive.

## Requirements

### Requirement: Replica rows and their resume checkpoint SHALL commit atomically

A persistence adapter that stores a resume position SHALL write the rows and the
checkpoint describing them in a single transaction. Writing them separately
leaves a window in which a crash produces rows whose resume position is unknown.

#### Scenario: Rows and checkpoint are written in one statement

- **GIVEN** a persistence adapter
- **WHEN** a value is written together with a resume checkpoint
- **THEN** exactly one write statement carries both the value and the checkpoint

#### Scenario: An existing store gains checkpoint storage without being dropped

- **GIVEN** a store created before checkpoints existed
- **WHEN** the adapter is constructed against it
- **THEN** checkpoint storage is added in place and existing rows are preserved

#### Scenario: A value written without a checkpoint reports none

- **GIVEN** a value written through the plain set path
- **WHEN** its checkpoint is read
- **THEN** no checkpoint is reported

### Requirement: A resume SHALL be refused unless the checkpoint describes the stored rows

Before resuming, a consumer SHALL confirm that the stored checkpoint describes
the stored rows. Where it cannot, the replica generation SHALL be rebuilt rather
than resumed, and stale rows removed rather than merged with newer data.

#### Scenario: Resume proceeds when generation and handle agree

- **GIVEN** stored rows and a checkpoint whose generation and shape handle match what the caller expects
- **WHEN** the resume is evaluated
- **THEN** the decision is to resume from that checkpoint

#### Scenario: Rows without a checkpoint force a rebuild

- **GIVEN** stored rows with no checkpoint
- **WHEN** the resume is evaluated
- **THEN** the decision is to rebuild, because the rows' position is unknown

#### Scenario: A checkpoint from an earlier generation forces a rebuild

- **GIVEN** a stored checkpoint whose generation precedes the caller's
- **WHEN** the resume is evaluated
- **THEN** the decision is to rebuild

#### Scenario: A changed shape handle forces a rebuild

- **GIVEN** a stored checkpoint whose handle differs from the one the server most recently issued
- **WHEN** the resume is evaluated
- **THEN** the decision is to rebuild, because offsets do not carry across handles

#### Scenario: A cold start is a rebuild, not a fault

- **GIVEN** no stored value
- **WHEN** the resume is evaluated
- **THEN** the decision is to rebuild, reported as absence rather than as an error

### Requirement: A change batch SHALL carry its transport resume position when one exists

A change batch delivered to a consumer SHALL carry the transport's resume
position for that batch where the transport defines one, and SHALL omit it where
it does not. An omitted position means the batch cannot be checkpointed; it does
not mean the beginning of the stream.

#### Scenario: A shape batch carries handle and offset

- **GIVEN** a batch of shape messages carrying offsets and a stream with a handle
- **WHEN** the batch is delivered
- **THEN** it carries the shape handle and the offset of its last message

#### Scenario: A local notification carries no resume position

- **GIVEN** a change originating from a local database notification rather than the shape stream
- **WHEN** it is delivered
- **THEN** it carries no resume position
