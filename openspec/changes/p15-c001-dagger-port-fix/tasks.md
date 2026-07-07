# Tasks — p15-c001-dagger-port-fix

- [x] In `dagger/codegen.ts` Stage 10 block: change healthz poll URL from `http://localhost:8080/healthz` to `http://localhost:28080/healthz`
- [x] In `dagger/codegen.ts` Stage 10 block: change `.withEnvVariable("GATEWAY_URL", "http://localhost:8080")` to `http://localhost:28080`
- [x] Verify no other `localhost:8080` references remain in the Stage 10 block
