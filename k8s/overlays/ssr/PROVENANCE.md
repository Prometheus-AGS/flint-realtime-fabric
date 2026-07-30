# SSR candidate provenance

- Source repository: local `flint-realtime-fabric`
- Baseline source SHA: `edbb21556b0b37e2d7431e3969bcdb0c62fd6b9c`
- License: MIT
- Build input: repository `Dockerfile`, no `CARGO_FEATURES`, `linux/amd64`;
  `node:24-slim@sha256:6f7b03f7c2c8e2e784dcf9295400527b9b1270fd37b7e9a7285cf83b6951452d`,
  `rust:1.85-bookworm@sha256:e51d0265072d2d9d5d320f6a44dde6b9ef13653b035098febd68cce8fa7c0bc4`,
  and
  `debian:trixie-slim@sha256:020c0d20b9880058cbe785a9db107156c3c75c2ac944a6aa7ab59f2add76a7bd`
- Candidate image: `ghcr.io/prometheus-ags/flint-realtime-fabric`
- Candidate digest: populated after the candidate build; an all-zero digest is
  deliberately rejected by `validate-overlay.sh`.
- Iggy: `iggyrs/iggy@sha256:68a314c1380be5a792a134f3bd346ded42bd49d9f7114c86f70b48fc85bc5272`
  (the immutable resolution of the repository's prior `latest` input on
  2026-07-30).
- Ory Keto: `oryd/keto:v0.14.0@sha256:c209da1c2f0f764f5790ef688ecb723141a84692dd6d9ab8c5bbefc3ead7838a`
  (upgraded from the compose file's obsolete v0.12 line).
- PostgreSQL: `postgres:16.14-alpine3.24@sha256:57c72fd2a128e416c7fcc499958864df5301e940bca0a56f58fddf30ffc07777`.
- AKS storage class: `managed-csi` (`disk.csi.azure.com`, expansion enabled),
  verified against the `ssr` context on 2026-07-30.

The minimum overlay contains only the FRF gateway, Iggy, Keto, Keto PostgreSQL,
migration, and tenant-wide seed. It does not render LiveKit, a media service,
CDC, federation bridges, SurrealDB, or an admin-UI workload.
Private GHCR pulls reference the pre-created `ghcr-pull` image pull Secret.
