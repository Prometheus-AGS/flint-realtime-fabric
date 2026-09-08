---
id: writing-an-adapter
title: Writing an adapter
sidebar_label: Writing an adapter
---

An adapter implements exactly one port against exactly one technology. This is
the most common way to extend FRF, and the rules are few but strict.

## The rules

1. **One port per adapter crate.** Not two. The
   [one recorded deviation](../theory/dependency-rule.md#a-recorded-deviation)
   is documented as a deviation and explicitly denied precedent.
2. **Never import another adapter.** If you need two capabilities, that is a
   composition, and composition happens in `frf-gateway`.
3. **Never import `frf-app`.** Adapters depend on `frf-domain` and `frf-ports`,
   nothing else from the workspace.
4. **No `unwrap()` or `expect()`.** Library crates return errors. Use
   `thiserror` locally and map to `PortError` at the boundary.
5. **Instrument every port method** with the mandated span name.

## The shape

Add a crate under `crates/`, kebab-case, named for the port and technology:
`frf-broker-iggy`, `frf-authz-keto`, `frf-store-redb`.

Its dependencies should be short:

```toml
[dependencies]
frf-domain = { path = "../frf-domain" }
frf-ports  = { path = "../frf-ports" }
# ... plus the one technology you are adapting
```

If that list grows an adapter crate, something is wrong.

## Implementing the trait

Match the instrumentation convention exactly — observability depends on the
names being predictable:

```rust
#[async_trait]
impl AuthzProvider for KetoAuthzProvider {
    /// Return `true` if the subject holds the relation on the object.
    ///
    /// # Errors
    ///
    /// Returns [`PortError::Transport`] if the HTTP request fails.
    /// Returns [`PortError::Serialization`] if the response body cannot be decoded.
    #[instrument(name = "port::AuthzProvider::check", skip(self, tuple), fields(relation = %tuple.relation))]
    async fn check(&self, tuple: &RelationTuple) -> Result<bool, PortError> {
        let key = CacheKey(
            tuple.subject.clone(),
            tuple.relation.clone(),
            tuple.object.clone(),
        );

        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }
```

Note what is *not* logged. The span records the relation but not the subject or
object — [never log](../theory/authorization.md#operational-rule) relation
tuples, tenant identifiers, or JWT payloads.

## Errors

Map your technology's failures onto `PortError` so callers can handle them
without knowing what is behind the port:

| Situation | Variant |
|---|---|
| Network, connection, timeout at transport level | `Transport` |
| The operation was understood and refused | `PermissionDenied` |
| Addressed a thing that does not exist | `NotFound` |
| Deadline exceeded | `Timeout` |
| Response could not be decoded | `Serialization` |
| The backing service returned an error of its own | `Upstream` |

`PortError` is `#[non_exhaustive]`, so matching on it requires a catch-all.

## Wiring it in

Composition is a match arm in `frf-gateway`, and it should stay that small:

```rust
match config.policy_engine {
    PolicyEngineMode::Cedar => { /* ... */ }
    PolicyEngineMode::None => {
        tracing::info!("action policy engine: no-op (all permitted)");
        Ok(BoxedPolicyProvider(
            Arc::new(NoOpPolicyProvider) as DynPolicyProvider
        ))
    }
}
```

If the runtime choice needs to be dynamic, use the erased wrapper for that port
(`DynMediaSignaler`, `BoxedPolicyProvider`) rather than making the gateway
generic over another parameter.

## Testing it

Your adapter's tests belong with the adapter and may use the real technology.
Use-case tests substitute an in-memory implementation of the port — that
substitution is [the whole point](../theory/dependency-rule.md#what-this-makes-possible)
of the seam, and it is why application rules can be verified in milliseconds
without Docker.

Run them locally. Never in CI.
