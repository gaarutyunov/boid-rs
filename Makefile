.PHONY: check test test-all test-embassy clippy fmt fmt-check clean help wasm check-embassy

# Default target
help:
	@echo "Available targets:"
	@echo "  make check         - Run all checks (tests, clippy, format) for std packages"
	@echo "  make test          - Run tests for boid-core and boid-wasm"
	@echo "  make test-all      - Run tests for all packages including workspace tests"
	@echo "  make test-embassy  - Check boid-embassy builds (requires nightly + RISC-V)"
	@echo "  make clippy        - Run clippy linter on std packages"
	@echo "  make fmt           - Format all code"
	@echo "  make fmt-check     - Check code formatting without modifying"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make wasm          - Build WASM package"
	@echo "  make check-embassy - Run all checks including embassy"

# Run all checks before committing (std packages only)
check: test clippy fmt-check
	@echo "✅ All checks passed!"

# Run all checks including embassy (requires nightly toolchain)
check-embassy: test-all test-embassy clippy fmt-check
	@echo "✅ All checks including embassy passed!"

# Run tests for standard environment packages
test:
	@echo "Running tests for boid-core..."
	@cargo test -p boid-core
	@echo "Running tests for boid-wasm..."
	@cargo test -p boid-wasm
	@echo "✅ Tests passed!"

# Run all workspace tests (now works because default-members excludes embassy)
test-all:
	@echo "Running workspace tests..."
	@cargo test --workspace
	@echo "✅ Workspace tests passed!"

# Check that embassy builds correctly (requires nightly + RISC-V target)
test-embassy:
	@echo "Checking boid-embassy builds..."
	@if ! rustup toolchain list | grep -q nightly; then \
		echo "❌ Nightly toolchain not installed. Run: rustup toolchain install nightly"; \
		exit 1; \
	fi
	@if ! rustup target list --installed --toolchain nightly | grep -q riscv32imc-unknown-none-elf; then \
		echo "❌ RISC-V target not installed. Run: rustup target add riscv32imc-unknown-none-elf --toolchain nightly"; \
		exit 1; \
	fi
	@cd boid-embassy && cargo +nightly check --target riscv32imc-unknown-none-elf
	@echo "✅ Embassy build check passed!"

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
