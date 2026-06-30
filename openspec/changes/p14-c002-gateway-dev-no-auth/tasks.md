# Tasks — p14-c002-gateway-dev-no-auth

- [ ] Read `crates/frf-gateway/src/config.rs` current content
- [ ] Read `crates/frf-gateway/src/routes/publish.rs` current content
- [ ] Read `crates/frf-gateway/src/routes/subscribe.rs` current content
- [ ] Add `dev_no_auth()` fn to config.rs under `#[cfg(feature = "dev-endpoints")]`
- [ ] Patch publish.rs to skip auth when `dev_no_auth()` returns true
- [ ] Patch subscribe.rs to skip auth when `dev_no_auth()` returns true
- [ ] Add `DEV_NO_AUTH: "true"` to gateway service env in compose.yml
- [ ] Run `cargo check -p frf-gateway` to verify compilation
