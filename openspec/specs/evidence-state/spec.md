# evidence-state Specification

## Purpose
TBD - created by archiving change p37-c005-evidence-not-loaded-state. Update Purpose after archive.

## Requirements

### Requirement: Unloaded evidence SHALL be distinguishable from absent evidence

A tally of evidence states SHALL carry whether the counts are known. All-zero
counts mean "loaded, and nothing matched"; they SHALL NOT be produced for a case
whose evidence has not yet arrived.

This SHALL NOT be achieved by adding a fourth evidence state. The state union is
closed at three members and is shared across languages; "not loaded" describes
the reader, not the evidence.

#### Scenario: A loaded empty case differs from an unloaded one

- **GIVEN** a case whose evidence has loaded and matched nothing
- **AND** a case whose evidence has not yet arrived
- **WHEN** each is tallied
- **THEN** the two tallies are not equal
- **AND** the first reports itself as loaded, the second as not loaded

#### Scenario: The evidence state union remains closed at three

- **WHEN** the evidence states are enumerated
- **THEN** exactly three are present

#### Scenario: Counts are reachable only when loaded

- **GIVEN** a tally that is not loaded
- **WHEN** its counts are requested
- **THEN** no counts are returned

### Requirement: Clinical text SHALL NOT be derived from unloaded counts

A statement about what a case is waiting on SHALL be produced only from counts
that are known to have arrived. Where they have not, no sentence SHALL be
returned, so that a caller cannot render a clinical claim about data it does not
have.

#### Scenario: An unloaded case yields no sentence

- **GIVEN** a case whose evidence has not arrived
- **WHEN** its outstanding work is described
- **THEN** no sentence is produced

#### Scenario: An unloaded case is never reported as ready

- **GIVEN** a case whose evidence has not arrived
- **AND** whose gate is affirmed
- **WHEN** its outstanding work is described
- **THEN** it is not reported as ready to draft

#### Scenario: A loaded case with nothing outstanding is still reported as ready

- **GIVEN** a case whose evidence has loaded with no gaps or voids
- **AND** whose gate is affirmed
- **WHEN** its outstanding work is described
- **THEN** it is reported as ready to draft

#### Scenario: Both outstanding states are still reported together

- **GIVEN** a loaded case with both gaps and voids outstanding
- **WHEN** its outstanding work is described
- **THEN** both are named in one statement
