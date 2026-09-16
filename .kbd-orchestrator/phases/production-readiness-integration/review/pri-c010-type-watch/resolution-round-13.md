# c010 adversarial review round 13 resolution

## Critical: alleged missing crate-root `IggyBroker` export

Disposition: rejected as a packet-context misread. The submitted
`crates/frf-broker-iggy/src/lib.rs` contains `pub use broker::IggyBroker;` at
line 10. The gateway integration import therefore resolves, as demonstrated by
strict all-target gateway Clippy and repeated compiled full-stack runs.

## Warning: malformed watch numbers silently default

Disposition: fixed. Generation, retention, and buffer-capacity environment
variables now use a fallible generic parser. A present malformed value fails
`GatewayConfig::from_env` with the variable name; only absence selects the
documented default. A focused test covers all three names. The checkpoint-key
field documentation was also corrected from HMAC to encryption terminology.

Strict gateway Clippy and the focused negative test pass.
