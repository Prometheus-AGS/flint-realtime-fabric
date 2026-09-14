# p13-c006 — Add `Makefile` with `baseline-wasm` target

## Summary

Add a `Makefile` at the workspace root with a `baseline-wasm` target that
automates the `.wasm-size-baseline` commit workflow described in c005.
Also adds convenience targets for common developer operations.

## File to create

- `Makefile` (workspace root)

## Specification

```makefile
.PHONY: help baseline-wasm cdc-smoke build test clippy fmt

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

baseline-wasm: ## Run Stage 6 and commit the WASM binary size baseline
	@echo "Running Dagger pipeline through Stage 6..."
	ENABLE_BENCH_STAGE=false ENABLE_INTEGRATION_STAGE=false \
	  dagger run ts-node dagger/codegen.ts
	@SIZE=$$(wc -c < sdks/ts/frf-wasm/frf_wasm_bg.wasm | tr -d ' '); \
	  echo "$$SIZE" > .wasm-size-baseline; \
	  echo "Baseline set: $$SIZE bytes"; \
	  git add .wasm-size-baseline; \
	  git commit -m "chore: update WASM binary size baseline ($${SIZE} bytes)"

cdc-smoke: ## Run CDC replication slot smoke test (requires running compose stack)
	bash scripts/smoke-cdc.sh

build: ## Build workspace in release mode
	cargo build --workspace --release

test: ## Run workspace tests
	cargo test --workspace

clippy: ## Run Clippy (CI-equivalent)
	cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic

fmt: ## Format all Rust code
	cargo fmt --all

compose-up: ## Start the full compose stack
	docker compose up -d

compose-down: ## Tear down the compose stack
	docker compose down

layer3-e2e: ## Run Stage 10 Layer 3 E2E (requires DinD / Docker host)
	ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
```

## Acceptance criteria

1. `make help` lists all targets with descriptions.
2. `make baseline-wasm` runs the Dagger pipeline, writes `.wasm-size-baseline`,
   and creates a git commit (when a DinD environment is available).
3. `make cdc-smoke` delegates to `scripts/smoke-cdc.sh` unchanged.
4. All targets use `PHONY` to avoid conflicts with files of the same name.
