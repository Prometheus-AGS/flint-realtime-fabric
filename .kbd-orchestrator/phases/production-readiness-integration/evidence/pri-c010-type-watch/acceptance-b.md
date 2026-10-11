# c010 acceptance B — race, recovery, lag, cancellation, and revocation

Captured: `2026-09-16`

Status: **PASS**

## Exact local proofs

| Boundary | Proof | Result |
|---|---|---|
| snapshot/subscribe race | `cargo test --offline -p frf-app --test entity_type_watch_race --locked` | PASS; a mutation inserted after the atomic projection snapshot captured broker offset 0 and before `subscribe` returned was replayed from offset 1 after `SnapshotComplete` |
| reconnect/resume | real run `20260916223916-49993`, `resume_lag_and_idle_revocation_are_explicit`, and the deterministic race test | PASS; snapshots without withheld rows resume after the atomic cursor, while disconnect after snapshot `Accepted` and snapshots with withheld rows replay from a safe earlier position instead of skipping undelivered rows |
| grant after empty snapshot | `cargo test --offline -p frf-app --test entity_type_watch_empty_snapshot --locked` | PASS; a zero-row fully filtered snapshot checkpoints immediately before its earliest withheld offset; with earlier history expired, resume after an object grant starts at offset 5 and delivers that row |
| expired cursor | `resume_lag_and_idle_revocation_are_explicit` | PASS; source retention rejection produces one terminal `HistoryExpired` resnapshot frame |
| advertised retention | broker unit tests plus gateway composition | PASS; Iggy topic expiry and `WatchAccepted.retention_seconds` derive from the same validated gateway value |
| checkpoint tamper/scope/epoch/generation | `cargo test --offline -p frf-app --lib watch_checkpoint --locked` | PASS; every mismatch fails closed with the specific recovery class |
| live source epoch change | `source_epoch_change_requires_resnapshot_before_delivery` | PASS; a matching authorized mutation from another epoch terminates with `SourceEpochChanged` before checkpoint or payload delivery |
| slow-client lag | `resume_lag_and_idle_revocation_are_explicit` and `stalled_initial_snapshot_becomes_terminal_lag` | PASS; live and initial-snapshot backpressure emit one terminal `Lagged { checkpoint_resumable: false }` through the independent control path |
| cancellation during awaited authorization | `cancellation_during_awaited_authorization_releases_subscription` | PASS; aborting while object authorization is blocked drops the source stream and its active count returns to zero |
| authority revocation | `resume_lag_and_idle_revocation_are_explicit`, `revocation_stops_initial_frames_before_enqueue`, `revoked_subscription_cannot_advance_an_out_of_scope_checkpoint`, and `revocation_during_object_authorization_preempts_live_delivery` | PASS; idle live, initial snapshot, filtered progress, and revocation during object authorization terminate before another payload or checkpoint frame |
| token expiry during authorization | `token_expiry_preempts_live_delivery_authorization` and `token_expiry_preempts_initial_frame_authorization` | PASS; expiry races awaited live and initial authorization, leaves the data channel empty, and emits one permission terminal |
| malformed typed payload | `cargo test --offline -p frf-gateway --lib entity_type_grpc_service::tests --locked` | PASS; malformed base64url fails with `DATA_LOSS` and delete data is stripped |
| terminal ordering | `cargo test --offline -p frf-app --lib entity_type_watch::stream --locked` | PASS; ten tests prove source errors follow queued frames, permission, expiry, and epoch recovery preempt protected data or progress, resnapshot is emitted once, and stalled snapshot delivery terminates |

## Mechanism checked

The snapshot comes from the durable typed projection with its cursor captured in
one store transaction. The service subscribes at `cursor + 1`, so a commit that
lands during snapshot preparation remains in durable broker history and cannot
fall through a broadcast timing gap. Live admission captures the broker head
and subscribes at the following offset. Resume decrypts and validates the opaque AES-256-GCM
checkpoint bound to the
subject, tenant, type, projection, epoch, version, and generation before opening
the source.

Every subscriber owns a bounded task and channel. Initial snapshot sends have a
bounded wait; live sends are non-blocking. Both paths report terminal lag rather
than waiting forever. Dropping the client stream aborts that task; dropping a
request future while snapshot authorization is awaited drops the already-open
source stream. The tests observe source resource counts returning to baseline
instead of inferring cleanup from a timeout. Object-authorized mutations also
receive a subscription-authority recheck immediately before enqueueing.

The source-bound real run used the generated tonic client against PostgreSQL,
Iggy, and SurrealDB and removed its owned containers, network, and volumes. No
CI result or empty-stream substitute is used.
