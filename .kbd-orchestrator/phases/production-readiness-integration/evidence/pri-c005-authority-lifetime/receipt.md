# c005 source-bound acceptance receipt

Status: **PASS**

Generated: `2026-09-15T14:16:44Z`

## Source identity

- Fabric base revision: `315f8494fafcad9c7f8f42b58b791607f79cd9d9`
- Product diff SHA-256: `260e9491192dfc1862fd93db7609db8183b6adbe01c1616dc862a6c050b8e1d9`
- Gate fixture revision: `de88f40dbfe77bc03203bafc2b64de9815fe27b2`
- Runtime runner SHA-256: `45edfedb36f41ac11231309c06dffb0567e87e9a7c1a08c14ff3bebb1547c692`
- Runtime test SHA-256: `db801bb137abdd4af3e5e31119c1c0d15f4342441ace1d70f06911c60c55e660`
- Machine receipt: `c005-authority-runtime.json`, exit 0, seven scenarios

## Local verification

- Owned Gate/Postgres/Keto/Fabric authority runner: 7 scenarios passed.
- Combined affected-package suite before the final cache-key delta: 150 passed,
  0 failed, 2 intentionally ignored owned-runner tests. The final cache delta's
  complete crate suite then passed 10 tests and strict Clippy.
- Protected shape timing suites: 10 passed, maximum observed cancellation
  1,500 ms against the 5,000 ms contract.
- Post-review Keto suite: 12 passed, including stale-refill fencing, tenant
  cache isolation, encoded tuple deletion and malformed-403 failure handling.
- Strict all-target Clippy for affected packages: passed with warnings denied.
- Rust format, Shellcheck, Bash syntax, Node syntax, touched-file 500-line cap
  and `git diff --check`: passed.
- `openspec validate pri-c005-authority-lifetime --strict --no-interactive`:
  passed.
- Tests ran only against owned local fixtures. No CI test run was used.

A duplicate aggregate rerun against the final delta was stopped without a test
result after `sccache` stalled for more than six minutes compiling unchanged
SurrealDB code on the external build volume. No compiler child or failure
diagnostic was present. This run is not counted as evidence; the prior aggregate,
fresh complete Keto suite and final owned integration run are the cited gates.

## Review disposition

Initial independent review: **BLOCK**, 2 critical, 1 warning, 0 suggestions.

- Completed task 5 only after saving this receipt and rerunning validation.
- Replaced the static database fixture password with a per-run random value.
- Replaced Keto deletion URL interpolation with encoded query serialization and
  added a reserved-character regression test.
- Bound cached Keto decisions to effective namespace, tenant and subject, with
  a regression proving identical tuples in different tenants miss the cache.
- Parse Keto's expected denial document before accepting a 403; malformed or
  contradictory responses now fail closed and are never cached as denials.
- Moved the cold integration build before service startup and credential minting
  after a failed receipt demonstrated token expiry during compilation.

## Compatibility

The subscription and Keto cache changes preserve existing public method
signatures. Replacing the public `JwksCache` alias with an age-bounded struct and
changing `new_cache()`'s return shape is a source-breaking public API change.
The workspace is pre-1.0, so c023 must select at least the next minor version and
publish migration notes for callers that accessed the cache directly. Cedar is
pinned to exact 4.11.2 as required by c003; a 4.12 migration remains a separate
compatibility change.
