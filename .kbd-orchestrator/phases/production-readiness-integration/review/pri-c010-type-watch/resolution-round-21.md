# c010 adversarial review round 21 resolution

## Warning: broker retention was only validated when CDC was enabled

Disposition: fixed. `ENTITY_WATCH_RETENTION_SECONDS` is passed into full-profile
broker construction regardless of CDC state, so its minimum is now validated
unconditionally before any adapter is built. The watch-only checkpoint,
generation, and buffer invariants remain conditional on CDC because the v2
service is omitted when CDC is disabled.

The new configuration regression sets retention to one second with CDC disabled
and proves startup rejects it with the variable name. Focused gateway test,
strict pedantic Clippy, and the repository file-size gate pass.
