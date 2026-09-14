# Tasks — p38-c006-dead-code-and-guard-classification

> **DONE** = dead code removed and every ignored test carries an explicit classification.
> **PROVEN** = each test classified "runnable" was actually run.

- [x] T1: Re-confirm `ws.rs` is unreferenced — no `mod ws;` anywhere, `ws_echo` has no
      caller. If either changed, STOP.
- [x] T2: Delete `ws.rs`, or document in-file why it is a deliberate stub.
- [x] T3: Confirm `fetch_and_cache` still has its two in-file callers and leave it alone.
- [x] T4: Classify each of the four `#[ignore]`d tests: runnable now, or permanently
      unrunnable locally with the reason in the ignore string.
- [x] T5: Run every test classified runnable and record the outcome. Assume none bites
      until observed failing.
- [x] T6: Sweep for other zero-caller `pub fn`s — include the defining file in the count,
      which the earlier sweep wrongly excluded.
