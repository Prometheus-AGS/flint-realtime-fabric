.PHONY: help baseline-wasm cdc-smoke build test clippy fmt check-file-size compose-up compose-down layer3-e2e local-integration

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
	  awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

baseline-wasm: ## Run the Dagger build pipeline and commit the WASM binary size baseline
	@echo "Running Dagger build pipeline..."
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

check-file-size: ## Enforce the 500-line file cap (CLAUDE.md / constraints.md R5)
	bash scripts/check-file-size.sh

compose-up: ## Start the full compose stack
	docker compose up -d

compose-down: ## Tear down the compose stack
	docker compose down

layer3-e2e: ## Run Layer 3 E2E locally against a local Docker Compose stack
	bash scripts/run-layer3-e2e.sh

local-integration: ## Prove authenticated gateway publish/subscribe against an owned Iggy fixture
	bash scripts/run-local-integration.sh
