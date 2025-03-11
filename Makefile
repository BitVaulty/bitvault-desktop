.PHONY: dev build release test core-test ui-test security-test clean lint check all wasm docs

# Primary development targets
dev:
	cargo run -p bitvault-app

build:
	cargo build --workspace

release:
	cargo build --workspace --release

# Test targets
test:
	cargo test --workspace

core-test:
	cargo test -p bitvault-core

ui-test:
	cargo test -p bitvault-ui

security-test:
	cargo test -p bitvault-core --features security_tests
	./scripts/test-security-boundary.sh

# Security validation
security-check:
	cargo audit
	cargo clippy --workspace -- -D warnings
	./scripts/check-security-boundaries.sh
	
# Cleanup
clean:
	cargo clean
	find . -type f -name "*.rs.bk" -delete

# Code quality
lint:
	cargo clippy --workspace -- -D warnings
	cargo fmt --all -- --check

check: lint security-check test

# Build wasm version (for future web interface)
wasm:
	cd bitvault-ui && trunk build

# Run in development mode with security boundary logging
dev-debug:
	BITVAULT_LOG=debug cargo run -p bitvault-app

# Run with process isolation verification
dev-secure:
	BITVAULT_VERIFY_ISOLATION=1 cargo run -p bitvault-app

# Generate documentation
docs:
	cargo doc --workspace --no-deps
	@echo "Documentation available at target/doc/bitvault_core/index.html"

# Build all variants
all: clean build release wasm

# Default target
.DEFAULT_GOAL := build 