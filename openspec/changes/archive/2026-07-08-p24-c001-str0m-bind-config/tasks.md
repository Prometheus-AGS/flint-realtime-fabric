# Tasks — p24-c001-str0m-bind-config

- [x] 1. Add `MediaConfig` + `StrOmTransport::with_config`; thread it through `create_session`→`negotiate` (bind `(bind_addr, udp_port)`, host candidate from `advertise_ip` or bound addr). Keep `new()`/`default()` = loopback. Split a `config.rs` sub-module if session.rs nears 500 lines.
- [x] 2. Rust tests (default = loopback candidate; configured advertise_ip/udp_port reflected) + wire `MediaConfig::from_env` into `main.rs` for sovereign. QA gate.
