# Tasks — p35-c001-ci-decode-job

- [x] 1. run-media-decode.sh: Linux-portable host address (172.17.0.1 gateway→JWKS on Linux; keep host.docker.internal on macOS; colima/HOST_NET stays gated).
- [x] 2. .github/workflows/decode-proof.yml: workflow_dispatch job on ubuntu-latest (build gateway image → decode entrypoint → assert framesDecoded>0 → upload artifacts); secrets in-job.
