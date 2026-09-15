# c007 source-bound acceptance receipt

Status: **PASS**

Captured: `2026-09-15T17:28:40Z`

## Source identity

- Fabric base revision: `5aaad523ad6803bd07992a6ca6e6d75238d968c9`
- Candidate artifact-set SHA-256:
  `6ea5790712525f25e9d7057a978fda8c0cb5d817b35472e5f2bad880a565f2df`
- Exact file hashes: `source-binding.sha256`
- Runtime receipt: `runtime-receipt.json`
- Pinned SDK source: Iggy `0.6.203`, fork revision
  `d34b9c96ad5a15334e06040d68fd7512beeba4c8`
- Pinned matching server: Iggy `0.4.214`, immutable image
  `iggyrs/iggy@sha256:2b2d79a5d58a35834cf69b1e2e05d2052c91d739d1d91449475bcd04de24cdc5`

The source binding covers every c007 product, fixture and runner file, including
untracked candidates; it does not rely on `git diff` omitting new files.
Unrelated modified workspace files are outside this receipt.

## Runtime acceptance

`scripts/run-broker-replay-integration.sh` exited 0 against its owned local
Compose project. Nine named receipt scenarios passed:

- crash before checkpoint replay;
- checkpoint and inclusive resume;
- independent consumers and receiver cancellation;
- producer restart position continuity;
- exact position mapping under concurrent publishers;
- seed before server restart;
- actual server restart;
- durable replay after restart;
- stable identity deduplication after restart.

The same run exercised physical retention expiry. The enabled one-second
cleaner deleted two closed test segments, and the adapter rejected the expired
explicit cursor with `resnapshot_required:`. Normal topics were inspected with
86,400-second retention. The fixture removed its owned containers and volume.

## Local quality checks

All commands ran locally against the candidate and exited 0:

| Gate | Command | Result |
|---|---|---|
| Broker unit/all-target tests | `cargo test --offline -p frf-broker-iggy --all-targets --locked` | PASS, 10 unit tests; live tests intentionally ignored outside their exact owned runner |
| Owned broker runtime | `scripts/run-broker-replay-integration.sh` | PASS, 9 receipt scenarios plus live expiry boundary |
| Focused strict lint | `cargo clippy --offline -p frf-broker-iggy --all-targets --locked -- -D warnings -W clippy::pedantic` | PASS |
| Workspace compile | `cargo check --offline --workspace --locked` | PASS |
| Workspace production lint | `cargo clippy --offline --workspace --lib --bins --locked -- -D warnings -W clippy::pedantic` | PASS |
| Dev-endpoint lint | `cargo clippy --offline --workspace --lib --bins --features frf-gateway/dev-endpoints --locked -- -D warnings -W clippy::pedantic` | PASS |
| Workspace test lint | `cargo clippy --offline --workspace --tests --locked -- -D warnings -W clippy::pedantic` | PASS |
| Rust format | `cargo fmt --all -- --check` | PASS |
| Shell lint/syntax | `shellcheck scripts/run-broker-replay-integration.sh` and `bash -n ...` | PASS |
| Compose preflight | `scripts/run-broker-replay-integration.sh --preflight-only` | PASS |
| File size and diff hygiene | touched-file line checks and `git diff --check` | PASS; largest touched file 473 lines |
| OpenSpec | `openspec validate pri-c007-broker-replay --strict --no-interactive` | PASS |

All runtime tests ran locally. No CI test invocation or result is evidence.

## Semver and rollout

No method signature or protobuf changes in c007. The existing pre-1.0
`LogBroker` behavior is clarified and corrected: publish and delivered envelopes
carry the same authoritative broker offset, explicit resume is inclusive, and
expired retained history requests resnapshot.
The compose override is an isolated qualification fixture and is not a
production deployment profile. c016 owns production capacity, retention and
recovery qualification; c023 owns release versioning and signoff.

## Review disposition

Initial independent review: **BLOCK**, 4 critical, 0 warnings, 0 suggestions.

- Three findings resulted from an overbroad packet: untracked evidence was
  absent and unrelated dirty workspace/KBD files were included. c007 now owns
  an explicit `files.txt`, and its exact candidate files are staged before the
  resolution packet so only this change is reviewed.
- The substantive finding correctly rejected the provisional publish return.
  `publish` now resolves the accepted stable message ID to its authoritative
  broker position, with live assertions for first, sequential, retry, restart
  and concurrent publishes.

The first scoped resolution review blocked with 1 critical and 1 warning. It
found a retention race between preflight and the first poll and the stale
proposal-only status. The subscription now verifies that the first delivery for
an explicit cursor exactly matches the request, otherwise it returns
`resnapshot_required:`; the proposal status reflects completed implementation.
The second scoped resolution review blocked with 1 critical and 1 warning. It
found that restart persistence used an administrative seek rather than the
public acknowledgement path, and that the implementation evidence still called
acceptance pending. The restart seed now acknowledges offset `1` through
`LogBroker::ack`; the latest owned run proves the acknowledged checkpoint,
inclusive replay, append continuity and deduplication across an actual server
restart. The implementation evidence now records the passing local acceptance.
All deterministic gates were rerun.

The fourth independent review passed with 0 critical findings, 0 warnings and
0 suggestions. It used a fresh-context gpt-5.5 judge against the gpt-6-astra
producer packet, with verified distinct models and strict sycophancy screening.
The reviewer checked every acceptance criterion plus correctness, regressions,
security, constraint compliance and silent-failure paths. c007 is converged.
