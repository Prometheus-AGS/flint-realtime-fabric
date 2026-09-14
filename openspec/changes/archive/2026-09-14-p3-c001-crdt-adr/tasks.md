# Tasks — p3-c001 crdt-adr

- [ ] **T1** Create `docs/decisions/` directory and write `docs/decisions/adr-001-crdt-engine.md`
  - Title: "ADR-001: CRDT Engine Selection — Loro over automerge-rs"
  - Sections: Status, Context, Decision, Consequences (positive + negative)
  - Status: Accepted
  - Decision summary: Loro 1.13.1 chosen; `loro-ffi` provides first-party UniFFI bindings
  - Verification: file exists; readable markdown

- [ ] **T2** Add Phase 3 workspace dependency entries to root `Cargo.toml`
  - Under `[workspace.dependencies]`, add:
    - `loro = { version = "1.13.1" }`
    - `loro-ffi = { version = "1.13.1" }`
    - `redb = { version = "4.1.0" }`
    - `surrealdb = { version = "3.1.5" }`
    - `uniffi = { version = "0.31.2" }`
  - Verification: `cargo metadata --no-deps` exits 0

- [ ] **T3** Add new crate members to `[workspace]` members list in root `Cargo.toml`
  - Entries to add: `"crates/frf-crdt"`, `"crates/frf-store-redb"`, `"crates/frf-store-surreal"`, `"crates/frf-ffi"`
  - Note: These directories do NOT exist yet — cargo will warn but not error at metadata level
  - Verification: `cargo metadata --no-deps 2>&1 | grep -c "failed to load"` equals 4 (expected — crates not yet created); exits 0 with warnings only

- [ ] **T4** Update `openspec/config.yaml` context block
  - Change `CRDT: Loro | automerge-rs (OPEN — decide before Phase 3)` to `CRDT: Loro 1.13.1 (ADR-001)`
  - Verification: `grep "ADR-001" openspec/config.yaml` exits 0
