# c010 adversarial review round 17 resolution

## Critical: fully filtered snapshot can skip rows after a later grant

Disposition: fixed and subsequently refined in round 18. A snapshot that
discloses zero authorized rows keeps its completion checkpoint before the
earliest in-scope withheld row instead of sealing the projection's global
cursor into the token. The live source still starts after the atomic projection
cursor, preserving the snapshot/subscribe race boundary; if the client
disconnects and resumes from the empty completion, retained history is replayed
through current authorization.

The new `fully_filtered_snapshot_resume_replays_rows_after_a_grant` regression
starts with a durable row denied by object authorization, observes a zero-row
snapshot, grants access, and resumes from the completion checkpoint. It proves
the previously filtered row is then delivered. Focused test and strict pedantic
Clippy both pass.
