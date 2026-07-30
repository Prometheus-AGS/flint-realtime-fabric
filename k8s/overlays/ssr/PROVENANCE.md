# SSR candidate provenance

- Source repository: local `flint-realtime-fabric`
- Baseline source SHA: `edbb21556b0b37e2d7431e3969bcdb0c62fd6b9c`
- License: MIT
- Build input: repository `Dockerfile`, no `CARGO_FEATURES`, `linux/amd64`;
  `node:24-slim@sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d`,
  `rust:1.94-bookworm@sha256:6ae102bdbf528294bc79ad6e1fae682f6f7c2a6e6621506ba959f9685b308a55`,
  and
  `debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd`
- Candidate image: `ghcr.io/prometheus-ags/flint-realtime-fabric`
- Candidate source SHA:
  `953ced6b41846066eb78b2dcd230865cae325339`
- Candidate OCI index digest:
  `sha256:4da5f05de5877e84c8dcaec5c78f7af02e00454138c5390f5d545c3aeae0f915`
- Candidate build:
  <https://github.com/Prometheus-AGS/flint-realtime-fabric/actions/runs/30541695972>
- Iggy: `iggyrs/iggy@sha256:68a314c1380be5a792a134f3bd346ded42bd49d9f7114c86f70b48fc85bc5272`
  (the immutable resolution of the repository's prior `latest` input on
  2026-07-30).
- AKS storage class: `managed-csi` (`disk.csi.azure.com`, expansion enabled),
  verified against the `ssr` context on 2026-07-30.

The minimum overlay contains only the FRF gateway and Iggy. It selects
`AUTHZ_BACKEND=verified-identity`: Gate JWT verification and in-process tenant
equality protect transport operations, while durable Sansaba data authorization
stays in the existing PostgreSQL RLS boundary. Keto remains an optional generic
platform adapter but is not deployed for Sansaba.
The overlay does not render Keto, LiveKit, a media service, CDC, federation
bridges, SurrealDB, or an admin-UI workload.
Private GHCR pulls reference the pre-created `ghcr-pull` image pull Secret.
