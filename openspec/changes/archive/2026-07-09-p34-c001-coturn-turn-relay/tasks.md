# Tasks — p34-c001-coturn-turn-relay

- [x] 1. coturn STUN-only → TURN relay in compose.sovereign.yml (realm + static-auth-secret from env + external-ip); MEDIA_ADVERTISE_IP=gateway bridge IP.
- [x] 2. Harness iceServers: add turn: URL + username/credential (both sender + receiver PCs), threaded via env; runner sets TURN_* env.
