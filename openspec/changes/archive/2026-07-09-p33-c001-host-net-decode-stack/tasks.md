# Tasks — p33-c001-host-net-decode-stack

- [x] 1. Add compose.host-net.yml: gateway+playwright+caddy+coturn network_mode:host; localhost addresses (GATEWAY_URL, STUN_URL, Caddy --to, MEDIA_ADVERTISE_IP=127.0.0.1).
- [x] 2. Runner layers compose.host-net.yml + sets host-net env (GATEWAY_JWKS_URL=http://localhost:<port>, GATEWAY_URL/STUN via the service env); bash + compose-config clean.
