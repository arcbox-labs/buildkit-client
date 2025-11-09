.PHONY: help init build test clean health proto-clean proto-init

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

init: proto-init ## Initialize project (pull proto files and build)
	cargo build

proto-init: ## Pull proto files from upstream
	./scripts/init-proto.sh

proto-clean: ## Clean proto files
	rm -rf proto

build: ## Build the project
	cargo build

test: ## Run tests
	cargo test

clean: ## Clean build artifacts
	cargo clean

health: build ## Check BuildKit daemon health
	cargo run -- health

# Docker compose shortcuts
up: ## Start BuildKit and registry
	docker-compose up -d

down: ## Stop BuildKit and registry
	docker-compose down

logs: ## Show BuildKit logs
	docker-compose logs -f buildkitd

# Development helpers
check: ## Run cargo check
	cargo check

fmt: ## Format code
	cargo fmt

clippy: ## Run clippy
	cargo clippy

run-local: build ## Test local build with example
	cargo run -- local --context ./examples/test-dockerfile --tag localhost:5000/test:latest

run-github: build ## Test GitHub build with example
	cargo run -- github https://github.com/tianon/gosu.git --tag localhost:5000/gosu:latest --git-ref master
