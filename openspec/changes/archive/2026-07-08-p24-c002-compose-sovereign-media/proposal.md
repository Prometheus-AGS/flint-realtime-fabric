# p24-c002-compose-sovereign-media

## Why

Phase-24 GAP-2: the production `compose.yml` gateway runs `SFU_MODE=hosted` and maps only TCP
`28080:8080` — no UDP media path. To exercise the decode proof (c004) a browser needs to reach
the sovereign gateway's media socket over UDP, with the advertised candidate IP set (c001 seam).

## What Changes

- New `compose.sovereign.yml` **override** (composed via `-f compose.yml -f compose.sovereign.yml`)
  that, for the gateway service: sets `SFU_MODE=sovereign`, maps a **fixed UDP port**
  (`MEDIA_UDP_PORT`, `<PORT>:<PORT>/udp`), and sets `MEDIA_BIND_ADDR=0.0.0.0` +
  `MEDIA_ADVERTISE_IP` (host-reachable, e.g. `host.docker.internal`).
- The production `compose.yml` default is **untouched** (stays `hosted`) — sovereign is opt-in via
  the override, mirroring the existing `compose.override.example.yml` / `compose.ci.yml` pattern.

## Impact

- New: `compose.sovereign.yml`.
- `compose.yml` unchanged. No gate flip. Enables the c004 live decode run.
