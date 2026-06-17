# p1-c002 — frf-app: Application Use-Cases

## Affected crates
- `crates/frf-app` (new — stub created by p1-c001)

## Dependency-rule impact
`frf-app` is Layer 1 (application). It imports ONLY `frf-domain` and `frf-ports`. It must NEVER import any adapter crate (`frf-broker-iggy`, `frf-authz-keto`, etc.). The compiler enforces this because adapter crates are not in `frf-app`'s `[dependencies]`.

## What this change does

Implements the application-layer use-cases that orchestrate port traits:

### `SubscribePipeline`
```
verify token (IdentityVerifier) → check subscribe permission (AuthzProvider)
  → broker.subscribe() → filter stream by per-event view permission → return EventStream
```

Generic over `<L: LogBroker, A: AuthzProvider, I: IdentityVerifier>`. Testable with `mockall` mocks.

### `PublishUseCase`
```
verify caller claims → broker.publish(envelope) → return assigned Offset
```

### `AppError`
`thiserror`-derived error enum covering: `Unauthorized`, `Forbidden`, `BrokerError(PortError)`, `IdentityError(PortError)`.

### Module layout
```
crates/frf-app/src/
├── lib.rs
├── error.rs          AppError
├── subscribe.rs      SubscribePipeline + SubscribeRequest
└── publish.rs        PublishUseCase + PublishRequest
```

## Phase 1 exit criterion satisfied
`frf-app` use-cases are tested with mockall mocks and pass with no real network.

## Non-goals
- Does not implement any port trait.
- Does not create database connections or HTTP clients.
- Does not wire into `frf-gateway` (that is p1-c007).
