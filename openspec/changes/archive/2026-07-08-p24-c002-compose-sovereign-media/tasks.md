# Tasks — p24-c002-compose-sovereign-media

- [x] 1. Write `compose.sovereign.yml`: override the gateway service with `SFU_MODE=sovereign`, `MEDIA_BIND_ADDR=0.0.0.0`, `MEDIA_ADVERTISE_IP=host.docker.internal`, `MEDIA_UDP_PORT=<PORT>`, and a `<PORT>:<PORT>/udp` mapping. Document the `-f compose.yml -f compose.sovereign.yml` invocation + that it does not touch the hosted default.
