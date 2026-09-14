# Tasks — p16-c002

- [x] Add tenant-equality check in publish.rs before broker append
- [~] Add tenant-equality check in subscribe.rs before broker subscribe
- [x] Add AppError variant / reuse Forbidden for tenant mismatch
- [x] Unit test: matching tenant passes
- [x] Unit test: mismatched tenant is rejected

## Notes

### Publish — implemented (the real H1 fix)

`PublishUseCase::execute` now rejects with `AppError::Forbidden` when the verified
JWT `tenant_id` != `req.envelope.channel.tenant_id`, BEFORE the Keto `publish` check or
the broker append. This closes the forged-cross-tenant-write path: a caller
authenticated for tenant A can no longer write an envelope stamped tenant B even if a
stray relation tuple would permit it. The verified JWT tenant is authoritative; the
envelope's channel tenant is caller-supplied and must match.

Reused the existing `AppError::Forbidden` variant (no new variant needed).

Tests (`crates/frf-app/tests/publish_usecase.rs`):
- `allows_publish_when_tenant_matches_channel` — matching tenant proceeds to broker.
- `rejects_publish_into_foreign_tenant_channel` — mismatched tenant is rejected; the
  broker and authz mocks have NO expectations, proving the guard short-circuits before
  either is consulted.

All 6 publish tests + full `frf-app` suite pass; `clippy --all-targets -D warnings
-W pedantic` clean; `fmt` clean.

### Subscribe — not applicable at the app layer (documented, not faked)

`SubscribeRequest` carries only a bare `ChannelId` (a UUID parsed from the WS query
string). The channel's owning tenant is never resolved — `LogBroker::subscribe` takes
`ChannelId` alone, and there is no channel→tenant lookup in this architecture. There is
therefore nothing to compare `claims.tenant_id` against at subscribe time without new
infrastructure (a channel registry keyed by tenant).

The tenant boundary on the READ path is already enforced by the existing per-event
Keto `view` check in `SubscribePipeline::execute` (`subscribe.rs:81-102`): every
delivered envelope is filtered by `check(subject, "view", envelope.id)` scoped to
`claims.tenant_id`. A subject cannot receive an event its tenant/subject is not granted
`view` on. Adding a redundant channel-tenant equality check would require a
channel→tenant resolution that does not exist and is out of scope for c002.

**Marked `[~]` (not applicable) rather than `[x]` or skipped** — an honest status. If a
channel registry is introduced later (e.g. alongside `frf-cli` channel management in
c015), a subscribe-time channel-tenant assertion should be revisited. Recorded as a
follow-up so it is not lost.
