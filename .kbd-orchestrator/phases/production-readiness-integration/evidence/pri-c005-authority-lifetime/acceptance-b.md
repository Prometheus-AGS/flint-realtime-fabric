# c005 acceptance B — protected delivery lifetime

Status: **PASS** for the c005 authority-lifetime boundary.

## Accepted clock

The c003 contract starts the clock at authoritative ASO membership commit,
durable session-denial commit, or verified expiry. For a direct Kratos
revocation, the clock starts when Gate observes the denial from Kratos or ASO.
The endpoint is the last server-produced protected frame, or cancellation that
prevents the next frame. Kernel/proxy buffers, transit and client receipt are
outside this server measure. The maximum is 5,000 ms and a new protected request
must then fail.

## Current-source measurements

Command:

`cargo test --offline -p frf-app --test shape_revocation --test shape_lease_timing --locked -- --nocapture`

Result: 10 passed, 0 failed, 0 ignored.

| Case | Start | Last-frame/cancellation result | New-request result |
|---|---|---:|---|
| membership/logout denial | authority changes to denied | 750 ms revalidation | forbidden |
| verified grant expiry | expiry instant | 1,000 ms | grant expired |
| authority service unavailable | first failed revalidation | 750 ms | continuation handle released |
| authority request stalls | first revalidation begins | 1,500 ms including the 750 ms check timeout | continuation handle released |
| upstream stalls | active lease starts | 1,000 ms | continuation handle released |
| client backpressure | active lease starts | 1,000 ms; buffered frames discarded | continuation handle released |
| direct Kratos/ASO denial observed by Gate | Gate-owned response consumer observes denial | immediate body drop | forbidden |

The observed maximum is 1,500 ms, leaving 3,500 ms below the accepted ceiling.
The tests assert the upstream body is dropped and its provisional continuation
handle is removed, so queued frames cannot be resumed after cancellation.

The timing sources are committed at Fabric revision
`41edee951f29de6d8c186f6c5ec6147780d134d8`. Their SHA-256 values are:

- `shape_revocation.rs`: `e9d1a33d41b257965bcf336c68bae3cc9e40983d1981342ea365ee0c107c1342`
- `shape_lease_timing.rs`: `7bc61bbceedd8d5e58150cd24694c83091f70ba066fd35179493d1741a1aef57`
- `support/shape_timing.rs`: `78892490d364fa05ecc82c4c87f3e78ffb58c6c0e6bfca944fa28c7251f90075`

## Other exposed lanes

Generic subscriptions now enforce token expiry with an independent timer, even
while idle. Before each protected event they recheck the channel grant and the
event object grant. Keto decisions live for at most one second; an authority
transport failure closes the stream. Cross-tenant and denied-object events are
discarded before delivery. The focused eight-scenario subscription suite and
the seven-scenario real Gate/Keto receipt cover these outcomes.
