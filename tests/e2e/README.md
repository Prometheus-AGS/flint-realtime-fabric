# E2E Smoke Tests

End-to-end smoke tests that verify all four SDK implementations can subscribe
and receive events via a running `frf-gateway`.

## Prerequisites

- `frf-gateway` running locally (`cargo run -p frf-gateway`)
- Apache Iggy running (or the LogBroker stub enabled)
- At minimum one of: `go`, `tsx` (Node), `dotnet`

## Running

```bash
# Start the gateway first
cargo run -p frf-gateway &

# Run smoke tests
./tests/e2e/smoke_test.sh

# With a non-default gateway URL
FRF_GATEWAY_URL=http://localhost:8080 ./tests/e2e/smoke_test.sh

# With auth token
JWT_TOKEN=<token> ./tests/e2e/smoke_test.sh
```

## Individual SDK tests

### TypeScript

```bash
FRF_GATEWAY_URL=http://localhost:4000 npx tsx tests/e2e/ts/smoke.ts
```

### Go

```bash
FRF_GATEWAY_URL=http://localhost:4000 go run -tags integration tests/e2e/go/main.go
```

### C#

```bash
FRF_GATEWAY_URL=http://localhost:4000 dotnet run --project tests/e2e/csharp/Smoke.csproj
```

## CI

These tests are excluded from default CI runs (`//go:build integration` in Go,
environment-gated in the shell script). They require a live gateway and
supporting infrastructure. Run them as a post-deploy sanity check.
