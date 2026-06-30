.PHONY: help baseline-wasm cdc-smoke build test clippy fmt compose-up compose-down layer3-e2e

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

baseline-wasm: ## Run Dagger pipeline through Stage 6 and commit the WASM binary size baseline
	@echo "Running Dagger pipeline (Stages 0–6 only)..."
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

layer3-e2e: ## Run Stage 10 Layer 3 E2E (requires DinD / Docker host with /var/run/docker.sock)
	ENABLE_INTEGRATION_STAGE=true dagger run ts-node dagger/codegen.ts
