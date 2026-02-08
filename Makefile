.PHONY: help test test-unit test-integration coverage lint fmt build run clean docker-build docker-run

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

test: ## Run all tests
	cargo test --verbose

test-unit: ## Run unit tests only
	cargo test --lib --verbose

test-integration: ## Run integration tests only
	cargo test --test '*' --verbose

coverage: ## Generate test coverage report
	cargo tarpaulin --out Html --output-dir coverage
	@echo "Coverage report generated in coverage/index.html"

coverage-fast: ## Generate optimized test coverage report for modified files
	./scripts/test_coverage_optimized.sh
	@echo "Fast coverage report generated in coverage-fast/index.html"

lint: ## Run clippy linter
	cargo clippy -- -D warnings

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt -- --check

fmt-fix: ## Format code and remove trailing whitespace
	find src -name "*.rs" -type f -exec sed -i 's/[[:space:]]*$$//' {} \;
	cargo fmt

build: ## Build release binary
	cargo build --release

build-dev: ## Build debug binary
	cargo build

run: ## Run in development mode
	cargo run

run-release: ## Run release binary
	cargo run --release

clean: ## Clean build artifacts
	cargo clean
	rm -rf coverage/

docker-build: ## Build Docker image
	docker build -t kusanagi:latest .

docker-run: ## Run Docker container
	docker run -p 8080:8080 kusanagi:latest

watch: ## Watch for changes and run tests
	cargo watch -x test

bench: ## Run benchmarks
	cargo bench

audit: ## Security audit
	cargo audit

update: ## Update dependencies
	cargo update

check: ## Quick compile check
	cargo check

all: fmt lint test build ## Run all checks and build
