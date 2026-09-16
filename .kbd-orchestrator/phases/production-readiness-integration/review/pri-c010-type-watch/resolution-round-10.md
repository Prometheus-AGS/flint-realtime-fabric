# c010 adversarial review round 10 resolution

## Critical: advertised retention can exceed Iggy retention

Disposition: fixed. The production gateway now constructs `IggyBroker` with the
same validated `ENTITY_WATCH_RETENTION_SECONDS` value supplied to the watch use
case and advertised in `WatchAccepted`. Topic creation and convergence of
existing topics both apply that configured duration. The adapter retains its
24-hour default constructor for independent consumers, while its configured
constructor rejects zero before connecting.

Unit tests cover the 24-hour default, a custom two-day expiry, and invalid zero.
Strict all-target Clippy for broker and gateway and all eleven broker unit tests
pass.
