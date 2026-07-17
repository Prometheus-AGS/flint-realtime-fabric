# Tasks — p36-c001-ice-fix-and-log-capture

- [x] T1: Fix gateway-log capture in `run-media-decode.sh` — copy `/tmp/p29-gateway.log` to `${GITHUB_WORKSPACE:-/tmp}/gateway.log` immediately after the existing log write on the failure path
- [x] T2: Set `MEDIA_ADVERTISE_IP: "gateway"` in `compose.sovereign.yml` gateway service env (replaces the env-var interpolation `${MEDIA_ADVERTISE_IP:-127.0.0.1}`)
- [x] T3: Override coturn `entrypoint` + `command` in `compose.sovereign.yml` to use `/bin/sh -c` shell expansion so `--external-ip=$(hostname -i | cut -d' ' -f1)` resolves at container start
- [x] T4: Verify `cargo check -p frf-media-str0m` still passes (no Rust code changed, but confirm)
- [x] T5: Push changes to `sovereign-sfu-decode-proof` branch (triggers CI decode-proof.yml)
