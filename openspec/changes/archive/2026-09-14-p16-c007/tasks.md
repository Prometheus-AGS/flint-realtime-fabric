# Tasks — p16-c007

- [x] Decide functional-vs-disabled for v1
- [x] If functional: supply Entities + a matchable policy set
- [x] If disabled: gate off with explicit log, no permit-all default
- [x] Test: decision path is not silently allow-all or deny-all

## Notes

### Decision: FUNCTIONAL (Cedar already works for action policies)

Investigation showed the audit's H3 ("allow-all / deny-all, structurally incapable of
real ABAC") was **overstated** for the current state:

- `policy.cedar` is an **explicit, matchable** action policy:
  `permit(principal, action == Action::"Publish"|"Subscribe", resource)`. It is not a
  blanket permit-all. Pre-existing tests prove it: `Publish` is permitted, `Delete` is
  **denied** by default, and a custom policy overrides.
- `PolicyEngineMode::None` → `NoOpPolicyProvider` already logs
  `"action policy engine: no-op (all permitted)"` at startup — the allow-all mode is
  **explicit and logged**, not silent (task 3 was already satisfied).

So Cedar governs mutation actions correctly for the action-level policies FRF uses.
Task 2 ("supply Entities + matchable policy set"): the bundled policy IS matchable
without entities (action-level, no attribute conditions); a full entity store for
attribute ABAC is deferred (documented).

### The real fix: no silent deny on evaluation errors

Previously, `is_permitted` collapsed anything-not-`Allow` into `Ok(false)` — so a broken
or attribute-requiring policy (which Cedar reports as an authorization *error*, since
there are no entities to resolve attributes against) would **silently deny** every
request with no signal. Now the engine inspects `response.diagnostics().errors()`: if
any exist it logs them (`tracing::error!`) and returns `Err(PolicyError::Evaluation)`.
An operator misconfiguration fails loudly instead of quietly denying.

The `Entities::empty()` scope limitation is now documented explicitly on the
`CedarPolicyEngine` type ("# Scope and limitations"): action-level policies work;
attribute-based ABAC needs an entity store and is out of scope until one exists.

### Tests (`crates/frf-policy-cedar/src/lib.rs`)

New: `attribute_requiring_policy_surfaces_error_not_silent_deny` — a policy with a
`when { principal.role == "admin" }` attribute condition returns
`Err(PolicyError::Evaluation)` rather than a silent deny. The 3 pre-existing tests
(permit Publish, deny Delete, custom override) still pass — proving the decision path
is neither silently allow-all nor silently deny-all.

### Verification

- `cargo clippy -p frf-policy-cedar --all-targets` → exit 0
- `cargo test -p frf-policy-cedar` → 4/4 pass
- `cargo fmt --check` → clean
