# c009 completion receipt

Status: **PASS; independently reviewed and ready to archive**

## Source-bound runtime

- Command: `./scripts/run-entity-projection-integration.sh`
- Run: `20260916132836-97391`
- Candidate artifact-set SHA-256:
  `bf36b30267b69ac6f60f433ce7e69ff157bb0047161dc5ff12b44627993a38de`
- Result: eight of eight scenarios passed; exit `0`
- Cleanup: owned Compose containers, network, and volumes removed
- Tracked receipt: `runtime-receipt.json`

## Local quality matrix

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `git diff --check` | PASS |
| `./scripts/test-deployment-profiles.sh` | PASS, including immutable images, negative inputs, TLS edge, and SSR rendering |
| `cargo test --offline -p frf-app --test entity_usecase --locked` | PASS, 6 tests |
| `cargo test --offline -p frf-gateway --lib entity_projector --locked` | PASS, 3 tests |
| `cargo test --offline -p frf-gateway --lib config::tests --locked` | PASS, 18 tests |
| `cargo test --offline -p frf-projection-surreal --locked` | PASS |
| `cargo check --offline -p frf-ports -p frf-app -p frf-projection-surreal -p frf-gateway --all-targets --locked` | PASS |
| `cargo check --offline -p frf-gateway --all-targets --locked` | PASS; proves the `#[async_trait]` `EntityStore` is dyn-compatible in gateway composition |
| Direct dependency audit | PASS; `frf-app` declares `futures-util`, and `frf-gateway` declares test-only `tokio-postgres` |
| `cargo clippy --offline -p frf-broker-iggy -p frf-projection-surreal -p frf-gateway --all-targets --locked -- -D warnings` | PASS |
| `openspec validate pri-c009-entity-projection --strict --no-interactive` | PASS |
| Hand-authored changed source/config files at most 500 lines | PASS; gateway `main.rs` is 484 lines |

All tests ran locally. No CI test result is used as evidence.

## Review corrections

The first review packet omitted intent-to-add files and produced two false
absence findings; it also prompted an explicit owned `String` conversion that
passes compilation. The complete-diff review identified and resolved the
reference Compose compatibility wording and a runbook service-name mismatch.
Later rounds resolved retained-backlog readiness, transaction cleanup,
non-finite payload handling, and the one-port broker capability boundary. The
After a revocation is detected during per-event authorization, the watch now
terminates on its next poll without polling an idle underlying source again.
Snapshot preparation cancels on every error, persisted operation names round
trip, and the final fresh-context review passed with no critical findings.
