.PHONY: check test clippy fmt fmt-check clean help

# Default target
help:
	@echo "Available targets:"
	@echo "  make check     - Run all checks (tests, clippy, format)"
	@echo "  make test      - Run tests for boid-core and boid-wasm"
	@echo "  make clippy    - Run clippy linter"
	@echo "  make fmt       - Format all code"
	@echo "  make fmt-check - Check code formatting without modifying"
	@echo "  make clean     - Clean build artifacts"
	@echo "  make wasm      - Build WASM package"

# Run all checks before committing
check: test clippy fmt-check
	@echo "✅ All checks passed!"

# Run tests for standard environment packages
test:
	@echo "Running tests for boid-core..."
	@cargo test -p boid-core
	@echo "Running tests for boid-wasm..."
	@cargo test -p boid-wasm
	@echo "✅ Tests passed!"

# Run clippy linter
clippy:
	@echo "Running clippy..."
	@cargo clippy -p boid-core -p boid-wasm -- -D warnings
	@echo "✅ Clippy passed!"

# Format all code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all
	@echo "✅ Code formatted!"

# Check formatting without modifying files
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check
	@echo "✅ Format check passed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@echo "✅ Cleaned!"

# Build WASM package
wasm:
	@echo "Building WASM package..."
	@cd boid-wasm && wasm-pack build --target web --out-dir www/pkg
	@echo "✅ WASM built!"
