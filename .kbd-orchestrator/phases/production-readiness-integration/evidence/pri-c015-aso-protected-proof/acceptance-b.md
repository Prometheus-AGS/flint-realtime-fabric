# Acceptance B — interruption, authorization and egress boundaries

Result: **Passed** for the selected browser PGlite `shape-only` topology.

## Interrupted bodies and egress

A real Chrome request received HTTP 200 and the first 256 bytes of a live Gate
shape response, then a browser `ReadableStream` error interrupted JSON parsing.
The mounted transport rejected the response without publishing SQL rows, a
checkpoint or PEM entities. Its next request was cold and hydrated the complete
authorized dataset.

The focused 99-test suite also projected an over-wide clinical row onto the
approved SQL target. Forbidden `member_id` and local-only `local_embedding`
canaries did not reach the generated statement.

## Authorization, expiry and revocation

The live composition passed ten checks: initial authorization, same-session
continuation, authenticated cross-identity handle denial, practice-scope
denial, projection denial, per-request grant revalidation, handle expiry to
HTTP 409 without shape headers, and authorized cold refetch with a fresh
handle/offset.

Current-source Fabric revocation tests passed 4/4 in the authoritative campaign.
The retained lease-timing receipt passed 10/10 with a 1,500 ms maximum against
the accepted 5,000 ms bound. Mounted logout independently revoked the live
Kratos session and the next protected request returned HTTP 401.

## Network and writer isolation

Gate is the only host-reachable protected shape endpoint. Electric and the FRF
facade expose zero host ports and occupy only the backend network. The client
segment reached Gate and received a session challenge but could not resolve or
reach FRF, Electric or the operator diagnostic. A host-browser-position probe
also could not reach Electric directly.

The 34-file production provider closure wires the FRF shape transport to the SQL
materializer and contains no generic entity-watch writer symbol. Runtime request
tracing observed shape requests only.

## File-size boundary

Every c015-modified ASO file is at most 500 lines. The campaign coordinator is
497 lines and the browser runner is 428 lines. Pre-existing oversized files that
were not modified by c015 remain tracked separately.

The current browser run recorded 591,396,864 bytes of renderer RSS growth,
above c014's 512 MiB memory threshold. It is retained as a nonblocking
observation here because c014 owns and already archived the memory acceptance
gate; all c015 functional, isolation and lifecycle checks passed.

## Source-bound evidence

| Artifact | SHA-256 |
|---|---|
| `campaign.json` | `6abf86af9336ece7e2fd32a6229f85d039e9b43b4cc20ff0415aa00cb00bea51` |
| `protected-composition.json` | `81d29178369fcffb8941e49744a05a9b1cad218df632ffc74e08c03cde7f1559` |
| `protected-topology.json` | `25f93143c2567423d8c73f6a6f9ae781f31a1122a7adc3779ce7506d40262563` |
| `task-4-revocation-timing.json` | `2074b485c4c5f213886a2cd0f8991ca928524ddcd532203f79e814f3c52c4ccc` |
| `task-4-mounted-browser-memory.json` | `9b08419b97cb587f9ad42a26e86dc18d0e390a3d9b8551a8a1683ef9103ab532` |
