# Tasks — p38-c001-iggy-handshake-diagnosis

> **DONE** = implemented and the finding is written down.
> **PROVEN** = the ignored suite actually completes (pass or fail) against a live server.

- [x] T1: Re-confirm the premise still holds — `iggyrs/iggy:latest` and the
      `k8s/overlays/ssr` pinned digest both still fail to accept a client from
      `cargo test -p frf-broker-iggy -- --ignored`. If either now works, STOP and re-assess.
- [x] T2: Capture the CLIENT side of the handshake with `RUST_LOG=iggy=trace` (or the
      crate's equivalent) against each server, bounded by `timeout` so nothing hangs.
- [x] T3: Identify where the exchange stalls — connect, login, or first command. Record the
      exact frame/step, not a version-mismatch inference.
- [x] T4: Decide the remedy from T3's evidence: pin a matching server digest, OR repin the
      client. If the answer is repin-the-client, STOP and report — do not proceed.
- [x] T5: Apply the remedy T4 selected — **pin the address family**, not a server digest.
      Fix `crates/frf-broker-iggy/tests/publish_subscribe.rs:32` and `:88`, which carry TWO
      defects in one string: `localhost` (resolves IPv6-first) and `guest:guest` (a user the
      server does not have — confirmed `Invalid credentials`). Use `127.0.0.1` and `iggy:iggy`.
      Also fix `crates/frf-cli/src/broker.rs:30` and `:47` (same `localhost` default) and
      `compose.ci.yml:57` (healthcheck uses `localhost`).
- [x] T6: Verify — `cargo test -p frf-broker-iggy -- --ignored` completes (pass or fail)
      rather than hanging, and the server logs a non-zero client count.
