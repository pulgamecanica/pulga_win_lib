.PHONY: help build test fmt clippy clean doc coverage window-demo mesh-demo

.DEFAULT_GOAL := help

help: ## Show this help
	@awk 'BEGIN {FS = ":.*##"} /^[a-zA-Z_-]+:.*##/ {printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## cargo build
	cargo build

test: ## cargo test
	cargo test

fmt: ## cargo fmt
	cargo fmt

clippy: ## cargo clippy
	cargo clippy

clean: ## cargo clean
	cargo clean

doc: ## cargo doc --open
	cargo doc --open

coverage: ## cargo tarpaulin --lib (window/example code needs a real display)
	cargo tarpaulin --lib --out Stdout --out Html

window-demo: ## Open an empty window (needs a display)
	cargo run --example window_demo

mesh-demo: ## Load+shade+spin the bundled 42-logo OBJ (needs a display)
	cargo run --example mesh_demo
