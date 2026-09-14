# p38-c001 — Diagnose the Iggy client/server handshake failure

## Summary

No Iggy server available to this project completes a handshake with the pinned client, so
`cargo test -p frf-broker-iggy -- --ignored` hangs instead of running. Under the
local-integration-only policy (AGENTS.md, CLAUDE.md) that makes **every** integration claim
in this crate unprovable — including the fix already landed for issue #2.

This change **diagnoses before it fixes**. The mechanism is undetermined; a corrective patch
written now would be a guess.

## Evidence

> **Revised at T1 (2026-09-14).** The original version of this section made two false
> claims, corrected below and on issue #6. They are left visible rather than erased,
> because T2–T4 must not rebuild on them.

**There is one server image, not two.** The digest pinned in `k8s/overlays/ssr/iggy.yaml:41`
and `iggyrs/iggy:latest` resolve to the *same* image id `sha256:f7c54e3042254` (created
2025-03-11, label `master`), and both report `version: 0.4.214`. The "0.8.13" in the original
table came from `/iggy/iggy --version` on that image — the **CLI binary** version, not the
server.

| Component | Version | Source |
|---|---|---|
| client | 0.6.203 | `Cargo.lock:4501`, GQAdonis fork `d34b9c96` |
| server (both `latest` and the "pinned" digest — identical image) | 0.4.214 | `compose.yml:58`, `k8s/overlays/ssr/iggy.yaml:41` |

**The client connects and a session is created.** The original claim that it "never
established a session" was a misreading of the periodic `Clients: 0` sysinfo counter. The
server log shows the opposite:

```
Accepted new TCP connection: 127.0.0.1:58494
Added TCP client with session: client ID: 4179809976
Created new session: client ID: 4179809976
```

**What still reproduces:** `cargo test -p frf-broker-iggy -- --ignored` hangs — exit 124
under a 60 s timeout — against this server. Confirmed twice at T1.

**Genuinely ruled out (re-established at T1, not inherited):**

- TCP reachability — connection accepted, logged server-side.
- Session establishment — server creates a session for this client.
- Credentials — `me`, a post-login command, succeeds with `-u iggy -p iggy`
  (Client ID 3610157041, User ID 1). An earlier check used `--credentials-username` /
  `--credentials-password`, which returns `Missing iggy server credentials` **and still
  exits 0** — a wrong-flag error on my part, not server behaviour.

**Therefore:** the stall is client-side, *after* TCP accept and session creation, against a
server that fully authenticates other clients over the same transport. The
protocol-version hypothesis is not dead, but the evidence originally cited for it
(a second, differing server) never existed.

## ROOT CAUSE — found at T2/T3 (2026-09-14)

**A name resolving IPv6-before-IPv4.** Not a version mismatch, not credentials, not
framing.

`getaddrinfo("localhost", 8090)` on this host returns **`::1` first**, `127.0.0.1` second,
and Docker publishes `*:8090` on both families. The client dials `[::1]:8090`; the server
accepts and creates a session; the connection is then reset before any frame completes
(`Connection reset by peer (os error 54)`). Because `max_retries: None` and
`reestablish_after: 5s`, the client reconnects and repeats **forever** — that infinite
reconnect loop is the "hang", not a block.

Single-variable proof:

| Connection string | Result |
|---|---|
| `iggy://iggy:iggy@localhost:8090` | reset → retry loop → timeout (exit 124) |
| `iggy://iggy:iggy@127.0.0.1:8090` | `connect()` → `Ok(Ok(()))` in **0.02 s** |

Server-side confirmation on the successful run:

```
Received a TCP request, length: 29
Received a TCP command: user.login|iggy|******, payload size: 29
Logging in user: iggy with ID: 1... → Logged in user: iggy with ID: 1.
Sent response with status: [0, 0, 0, 0]
```

For the IPv6 attempts the server logged accept + session but **zero**
`Received a TCP request` lines — the reset happens between session creation and the first
complete frame read.

**This is a repeat, not a novelty.** `phase-36`'s reflection records the same class as its
most expensive finding: *"Two of the four were the same root cause — a name resolving
IPv6-before-IPv4 — in two different places… it is invisible in every log: the gateway says
it is listening, the port says it is published, and the request still fails."* One of those
two was `GATEWAY_URL` using `localhost`.

**Second, independent defect found in the same lines.** `publish_subscribe.rs:32` and `:88`
default to `iggy://guest:guest@localhost:8090`, but the server's user is `iggy`. Those tests
would have failed on credentials even over IPv4.

**T4 decision (evidence-determined):** the remedy is neither pinning a server digest nor
repinning the client — it is pinning the **address family** at each call site. This does
**not** trip the stop-and-report gate, which covers repinning the client crate only.

## T5 — fix applied

| Site | Change |
|---|---|
| `crates/frf-broker-iggy/tests/publish_subscribe.rs:32,:88` | `guest:guest@localhost` → `iggy:iggy@127.0.0.1` (both defects) |
| `crates/frf-cli/src/broker.rs:30,:47` | `localhost` → `127.0.0.1` in the `--connection` default |
| `compose.yml:66`, `compose.ci.yml:57` | healthcheck `--server-address localhost` → `127.0.0.1` |

No `localhost:8090` remains in any iggy connection or healthcheck context.
`compose.yml:66` was missed on the first pass and fixed on a follow-up sweep.

Diagnostic scaffolding added during T2 (a `zz_connect_probe.rs` test and a
`tracing-subscriber` dev-dependency) was removed, and `Cargo.lock` reverted to HEAD.

## T6 — verified

```
running 2 tests
test a_subscriber_knowing_only_the_well_known_id_receives_published_events ... ok
test publish_then_subscribe_receives_message ... ok
test result: ok. 2 passed; 0 failed; finished in 0.06s
```

Run from the prebuilt binary with **`IGGY_TEST_CONNECTION_STRING` deliberately unset**, so
the fixed default is what was exercised rather than a shell override. Server logged 2
accepted TCP connections and 2 created sessions. The default (non-ignored) suite is
unaffected: 1 unit test passes, both integration tests correctly skip.

**On the exit criterion's exact wording.** T6 asked for "a non-zero client count". The
sysinfo `Clients:` field still reads 0, because it is a *sampled gauge* on a 60 s tick and
the run lasted 0.06 s. The criterion is met in substance — 2 accepts, 2 sessions, 2 passing
tests — but not as literally worded. That wording was mine, and it derived from the same
misreading of `Clients:` that produced the false "never established a session" claim
corrected earlier in this proposal.

**What this does NOT establish.** `a_subscriber_knowing_only_the_well_known_id_receives_published_events`
passing does not close issue #2. That guard must be observed to **fail** under sabotage
before it proves anything, which is `p38-c002`'s scope. A guard that has only ever passed
is a hypothesis.

**Undetermined and not claimed:** whether the compose service-to-service string
`iggy://iggy:iggy@iggy-server:8090` is affected. Two attempts to resolve `iggy-server`
inside the Docker network returned no output, so production wiring is **untested**, not
proven safe.

## Why this is independent

It depends on nothing. Everything else in the phase either depends on it (c002, and c006's
classification of the Iggy-blocked tests) or touches disjoint trees (c003, c004, c005).

## Scope

Observe first: capture the client side of the handshake (`RUST_LOG=iggy=trace` or
equivalent) and determine where it stalls. Only then choose between pinning a server digest
that matches the fork, or repinning the client.

**Stop-and-report condition:** if the diagnosis points at repinning the *client*, stop and
report rather than proceeding — that widens scope well beyond this phase.

Secondary, once the cause is known: replace the floating `iggyrs/iggy:latest` tag with a
digest so this breakage cannot change without a repo change. Note `26e4dfc` is titled
"align FRF broker with **pinned** Iggy server" while no pin exists in either compose file.

## Non-goals

- Fixing issue #2's receipt proof — that is c002 and depends on this.
- Authoring an `event-spine` capability delta. This change produces a *finding*; the
  behaviour change belongs to whatever fix follows it.

## Files

`compose.yml`, `compose.ci.yml`, `crates/frf-broker-iggy/tests/publish_subscribe.rs`
(diagnostic instrumentation only), plus a written finding.
