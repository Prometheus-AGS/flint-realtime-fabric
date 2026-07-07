# docs-accuracy (delta)

## ADDED Requirements

### Requirement: Docs MUST match the implemented surface

Documentation MUST reflect what the code actually does: the API reference MUST show
EntityService and AuthzService as live (not proto-only), source doc-comments MUST NOT
reference methods that do not exist, and the CHANGELOG MUST record phase-17 completions
and refreshed deferrals.

#### Scenario: API reference shows all six services live

- **WHEN** a reader consults the API reference server-status table
- **THEN** EntityService and AuthzService are marked live, not proto-only

#### Scenario: no doc-comment references non-existent methods

- **WHEN** the str0m signaler's doc-comment is read
- **THEN** it does not reference `process_offer` / `process_ice` (which do not exist) and
  honestly states the WebRTC media plane is deferred
