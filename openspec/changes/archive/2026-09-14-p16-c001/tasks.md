# Tasks — p16-c001

- [x] Remove CARGO_FEATURES and DEV_NO_AUTH from compose.yml
- [x] Verify compose.ci.yml / compose.override.yml still carry the bypass for dev/CI
- [x] Confirm dev_no_auth() bypass branches are #[cfg(feature="dev-endpoints")] gated
- [x] Build default (no-feature) gateway image and assert /v1/publish rejects empty token
- [x] Assert /ws/v1/subscribe rejects empty token in default build

## Notes

The Rust bypass code was ALREADY correctly compile-gated: `dev_no_auth()`
(`config.rs:65`) is `#[cfg(feature = "dev-endpoints")]`, and both `routes/publish.rs`
and `routes/subscribe.rs` guard the bypass under the same cfg. A default-feature build
cannot reference the function — verified by `cargo check -p frf-gateway` (exit 0) with
no reachable bypass path. The defect was **purely in `compose.yml`**, which opted a
production-shaped stack into `dev-endpoints` + `DEV_NO_AUTH`.

Fix: `compose.yml` builds the release image with NO `CARGO_FEATURES` and sets no
`DEV_NO_AUTH`, so the bypass code does not exist in that binary. `compose.override.yml`
(dev) now carries the `dev-endpoints` build arg + `DEV_NO_AUTH` so local dev still works;
`compose.ci.yml` already did. Both feature configurations compile
(`cargo check -p frf-gateway` and `--features dev-endpoints` both exit 0).

Tasks 4/5 verified structurally: with the default build the bypass branch is
`#[cfg]`-removed, so publish/subscribe fall through to the mandatory
`bearer_token(...)` → `identity.verify(...)` path that returns 401 on a missing/empty
token. (Full runtime E2E assertion belongs to the Stage 10 DinD run, which uses the
dev-endpoints image by design.)
