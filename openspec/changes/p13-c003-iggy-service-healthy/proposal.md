# p13-c003 — Change iggy `depends_on` to `service_healthy`

## Summary

In `compose.yml`, the `gateway` service declares:

```yaml
depends_on:
  iggy-server:
    condition: service_started
```

`service_started` means Docker only waits for the iggy container to exist,
not for iggy to be ready to accept connections. The iggy image has a
healthcheck defined (`iggy me` TCP probe), but its `condition` is not
used by the gateway's `depends_on`.

If iggy is still initialising when the gateway's `IggyBroker` attempts to
connect, the `ensure_channel` call and subsequent publish attempts will fail
with a connection error, causing Stage 10 to produce misleading test failures
that are not application bugs.

## File to change

- `compose.yml` — gateway `depends_on.iggy-server.condition`

## Specification

```yaml
# Change:
  gateway:
    depends_on:
      iggy-server:
        condition: service_started
      keto:
        condition: service_started
      postgres:
        condition: service_healthy

# To:
  gateway:
    depends_on:
      iggy-server:
        condition: service_healthy
      keto:
        condition: service_started
      postgres:
        condition: service_healthy
```

Note: `keto` healthcheck is defined in compose.yml (wget probe), but the
gateway does not block on Keto at startup (lazy connection). Changing keto to
`service_healthy` is optional; iggy is the priority because the gateway
connects eagerly at startup.

## Acceptance criteria

1. `docker compose up -d gateway` waits for the iggy healthcheck to pass
   before starting the gateway container.
2. The iggy healthcheck (`iggy me` TCP probe, 10s intervals, 10 retries,
   10s start_period) must report `healthy` before the gateway starts.
3. No regression in compose startup ordering for the remaining services.
