# Tasks — p23-c002-keto-view-check-on-join

- [x] 1. Add `authz: Option<Arc<dyn AuthzProvider>>` to `MediaTransportBridge`; add a `with_authz` builder; on `RoomJoin`, `check(subject="view", object=room_id)` before `join_room` — deny/error ⇒ skip join (log, no membership). Wire `KetoAuthzProvider` into the bridge in `main.rs`.
- [x] 2. Tests: authorized-join admitted (fan-out reaches it); unauthorized-join rejected (not in membership, no fan-out); cross-tenant join yields no fan-out. Use a stub `AuthzProvider` (deny/allow) to avoid a live Keto.
