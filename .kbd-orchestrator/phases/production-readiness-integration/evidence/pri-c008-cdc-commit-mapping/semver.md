# c008 semver impact

`CdcConfig` gains source-epoch and enrollment fields plus builder methods.
`TableEnrollment`, `TenantMode`, `CdcMutation` and its typed source/value models
become public from the pre-1.0 `frf-postgres-cdc` crate. `CdcError` changes its
decode failure surface to enrollment and transaction failures.

This is an additive product capability with breaking source impact for callers
that construct the public `CdcConfig` struct literally or exhaustively match
the public error enum. The workspace remains at its current development version
during the phase; c023 release signoff must record this as at least a minor
pre-1.0 release increment and publish migration notes for literal constructors.
