# c010 adversarial review round 7 resolution

## Critical: live delivery accepts a changed source epoch

Disposition: fixed. Live and resumed deliveries now compare the mutation source
epoch with the configured watch source epoch before checkpoint issuance, scope
filtering, authorization, or payload construction. A mismatch terminates through
the independent control path with
`ResnapshotRequired(SourceEpochChanged)` and preempts buffered protected frames.

The focused regression constructs an otherwise authorized matching mutation
from a different epoch and proves the result is the terminal resnapshot frame.
All four stream ordering, backpressure, authorization, and epoch tests pass.
