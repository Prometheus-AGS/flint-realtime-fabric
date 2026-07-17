# p36-c001 — ICE candidate fix + gateway-log capture

## Summary

Two targeted fixes that make the next CI run diagnosable AND likely to produce a completing ICE
pair on `ubuntu-latest`:

1. **Gateway-log capture** (`scripts/run-media-decode.sh`): copy the already-captured
   `/tmp/p29-gateway.log` to `${GITHUB_WORKSPACE:-/tmp}/gateway.log` so the CI artifact upload
   finds it at the expected workspace path.

2. **`MEDIA_ADVERTISE_IP=gateway`** (`compose.sovereign.yml`): replace the hardcoded
   `172.17.0.1` (docker0 host bridge — unreachable from the Compose bridge) with the service
   name `gateway`. `crates/frf-media-str0m/src/config.rs:resolve_advertised_ip()` resolves
   hostnames via `to_socket_addrs()` at negotiate time; Docker injects service names into
   `/etc/hosts` of all containers in the Compose network, so `gateway` resolves to the
   container's own bridge IP (`172.18.0.x`). The Playwright container (same bridge) can reach
   this address directly.

3. **coturn `--external-ip` shell expansion** (`compose.sovereign.yml`): replace the
   command array with an `entrypoint: ["/bin/sh", "-c"]` + inline shell string so
   `$(hostname -i | cut -d' ' -f1)` expands to coturn's own bridge IP at startup. This
   fixes the relay candidate, which currently points to `172.17.0.1` (same wrong address).

## Root cause (from assessment)

All Compose services share the project bridge (`172.18.0.0/16`). `172.17.0.1` is the host's
`docker0` gateway — a different bridge, unreachable from within the Compose network. Both the
host ICE candidate and the TURN relay candidate currently advertise this unreachable address,
so no candidate pair survives connectivity checks → `ice=checking` forever.

## Security notes (S1)

- No secrets added or committed. `TURN_SECRET` is still env-sourced.
- `MEDIA_ADVERTISE_IP=gateway` is a Docker service name, not an IP — no exposure.
- No changes to JWT, JWKS, or auth flow.

## Files

| File | Change |
|---|---|
| `scripts/run-media-decode.sh` | Add `cp /tmp/p29-gateway.log "${GITHUB_WORKSPACE:-/tmp}/gateway.log"` after existing log write on failure path |
| `compose.sovereign.yml` | gateway env: `MEDIA_ADVERTISE_IP: "gateway"` (was `${MEDIA_ADVERTISE_IP:-127.0.0.1}`) |
| `compose.sovereign.yml` | coturn: add `entrypoint: ["/bin/sh", "-c"]`; rewrite `command` as inline shell string with `$(hostname -i \| cut -d' ' -f1)` for `--external-ip` |
