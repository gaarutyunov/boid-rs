.PHONY: check test test-all test-embassy clippy fmt fmt-check clean help wasm check-embassy

# Default target
help:
	@echo "Available targets:"
	@echo "  make check         - Run all checks (tests, clippy, format) for std packages"
	@echo "  make test          - Run tests for boid-core and boid-wasm"
	@echo "  make test-all      - Run tests for all packages including workspace tests"
	@echo "  make test-esp32    - Check boid-esp32 builds (requires ESP toolchain)"
	@echo "  make clippy        - Run clippy linter on std packages"
	@echo "  make fmt           - Format all code"
	@echo "  make fmt-check     - Check code formatting without modifying"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make wasm          - Build WASM package"
	@echo "  make check-esp32   - Run all checks including esp32"

# Run all checks before committing (std packages only)
check: test clippy fmt-check
	@echo "✅ All checks passed!"

# Run all checks including esp32 (requires ESP toolchain)
check-esp32: test-all test-esp32 clippy fmt-check
	@echo "✅ All checks including esp32 passed!"

# Run tests for standard environment packages
test:
	@echo "Running tests for boid-core..."
	@cargo test -p boid-core
	@echo "Running tests for boid-wasm..."
	@cargo test -p boid-wasm
	@echo "✅ Tests passed!"

# Run all workspace tests (now works because default-members excludes esp32)
test-all:
	@echo "Running workspace tests..."
	@cargo test --workspace
	@echo "✅ Workspace tests passed!"

# Check that esp32 builds correctly (requires ESP Rust toolchain)
test-esp32:
	@echo "Checking boid-esp32 builds..."
	@if ! rustup toolchain list | grep -q esp; then \
		echo "❌ ESP toolchain not installed. Install with:"; \
		echo "   cargo install espup && espup install"; \
		echo "   Then source: . $$HOME/export-esp.sh"; \
		exit 1; \
	fi
	@if [ ! -f "$$HOME/export-esp.sh" ]; then \
		echo "❌ ESP environment not set up. Run:"; \
		echo "   espup install"; \
		echo "   . $$HOME/export-esp.sh"; \
		exit 1; \
	fi
	@cd boid-esp32 && cargo +esp check --target xtensa-esp32s3-none-elf
	@echo "✅ ESP32 build check passed!"

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
