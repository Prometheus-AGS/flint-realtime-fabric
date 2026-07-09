# p24-c001-str0m-bind-config

## Why

Phase-24 G2 (a real decoded frame across a browser↔gateway ICE path) is blocked by GAP-1:
`StrOmTransport` binds `UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))` — a loopback, ephemeral socket
— and advertises a `127.0.0.1:<random>` host candidate. A browser in another container/process
cannot reach that. `StrOmTransport::new()` takes no config, so there is no seam for a bindable
address, an advertised candidate IP, or a fixed UDP port. This change adds that seam.

## What Changes

- New `MediaConfig { bind_addr: IpAddr, advertise_ip: Option<IpAddr>, udp_port: u16 }` (in
  `frf-media-str0m`) + `StrOmTransport::with_config(MediaConfig)`. `new()`/`default()` keep the
  loopback-ephemeral behaviour (tests unaffected).
- Thread the config from `StrOmTransport` through `create_session` → `negotiate`: bind
  `(bind_addr, udp_port)`; build the host candidate from `advertise_ip` (falling back to the
  bound local address when absent).
- `frf-gateway/src/main.rs`: for `SFU_MODE=sovereign`, build `MediaConfig` from env
  (`MEDIA_BIND_ADDR` default `0.0.0.0`, `MEDIA_ADVERTISE_IP`, `MEDIA_UDP_PORT`) and use
  `with_config`.

## Impact

- `crates/frf-media-str0m/src/session.rs` (+ a small `config.rs` if it keeps files <500 lines).
- `crates/frf-gateway/src/main.rs` + config (env vars).
- Rust tests: default config = loopback host candidate; a configured `advertise_ip` appears in the
  advertised candidate; a configured `udp_port` is used.
- No gate flip. Enables c002–c004 (the live path).
