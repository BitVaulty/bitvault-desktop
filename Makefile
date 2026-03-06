.PHONY: dev build release test clean lint check security-check all docs

# Primary development targets
dev:
	cargo run

build:
	cargo build

release:
	cargo build --release

# Test targets
test:
	cargo test

# Security validation
security-check:
	cargo audit
	cargo clippy -- -D warnings

# Cleanup
clean:
	cargo clean
	find . -type f -name "*.rs.bk" -delete

# Code quality
lint:
	cargo clippy -- -D warnings
	cargo fmt --all -- --check

check: lint security-check test

# Build wasm version (optional)
# wasm:
# 	trunk build

# Run in development mode with security boundary logging
dev-debug:
	BITVAULT_LOG=debug cargo run

# Run with process isolation verification
dev-secure:
	BITVAULT_VERIFY_ISOLATION=1 cargo run

# Generate documentation
docs:
	cargo doc --no-deps
	@echo "Documentation available at target/doc/bitvault_app/index.html"

# Build all variants
all: clean build release

# Default target
.DEFAULT_GOAL := build
