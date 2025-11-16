# Contributing to Boid Simulation

Thank you for your interest in contributing to this project! This guide will help you set up your development environment and understand the contribution workflow.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for WASM development)
- For embedded development (ESP32-S3 Sense):
  - ESP Rust toolchain: `cargo install espup && espup install`
  - Environment setup: `. $HOME/export-esp.sh` (add to your shell profile)
  - espflash: `cargo install espflash`

## Development Workflow

### Before Committing

**IMPORTANT:** Always run the following checks before committing code:

```bash
# Quick check - runs all standard environment checks
make check

# Or if you have ESP toolchain (for esp32):
make check-esp32

# Or manually run each check:
cargo test --workspace                   # Run all workspace tests
cargo clippy -p boid-core -p boid-wasm -- -D warnings  # Run linter
cargo fmt --all -- --check              # Check formatting
```

If any of these fail, fix the issues before committing.

### Workspace Configuration

The `boid-esp32` crate is **excluded from the workspace** entirely:
- It requires ESP Rust toolchain (Xtensa architecture) incompatible with standard builds
- `cargo test --workspace` runs successfully without trying to build esp32
- Build esp32 separately: `cd boid-esp32 && cargo build`
- Use `make test-esp32` to check the esp32 crate builds correctly

### Fixing Issues

#### Formatting Issues
```bash
cargo fmt --all
```

#### Clippy Warnings
Read the clippy output and fix the suggested issues. Common fixes:
- Remove unused variables
- Add missing documentation
- Simplify complex expressions
- Use idiomatic Rust patterns

#### Test Failures
- Read the test output carefully
- Fix the failing tests or update them if the behavior intentionally changed
- Ensure all tests pass before committing

### Making Changes

1. **Create a branch** for your feature or bugfix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** in the appropriate crate:
   - `boid-core/` - Core algorithm changes
   - `boid-wasm/` - WebAssembly frontend changes
   - `boid-esp32/` - Embedded system changes

3. **Write tests** for new functionality:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_your_feature() {
           // Your test here
       }
   }
   ```

4. **Run all checks**:
   ```bash
   make check
   ```

5. **Commit your changes** with a descriptive message:
   ```bash
   git add .
   git commit -m "Add feature: description of your changes"
   ```

6. **Push and create a pull request**:
   ```bash
   git push origin feature/your-feature-name
   ```

## Project Structure

### boid-core (no_std compatible)
The core boid algorithm that works in both standard and embedded environments.

**Key files:**
- `src/lib.rs` - Main implementation
- `Cargo.toml` - Dependencies (note the `std` feature)

**Testing:**
```bash
# Test with std
cargo test -p boid-core

# Check no_std compatibility
cargo check -p boid-core --no-default-features
```

### boid-wasm
WebAssembly frontend for browser-based simulation.

**Key files:**
- `src/lib.rs` - WASM bindings
- `www/` - Web assets

**Testing:**
```bash
cargo test -p boid-wasm
cargo check -p boid-wasm --target wasm32-unknown-unknown
```

### boid-esp32
Embedded implementation for ESP32-S3/C3/C6.

**Key files:**
- `src/main.rs` - Main application
- `src/display.rs` - Display driver
- `src/rng.rs` - Random number generator

**Note:** This crate requires special build configuration and can't be tested in the standard environment. See [boid-esp32/README.md](boid-esp32/README.md) for details.

## Code Style

We follow standard Rust conventions:
- Use `rustfmt` for consistent formatting
- Follow Rust naming conventions (snake_case for functions, PascalCase for types)
- Add documentation comments for public APIs
- Keep functions focused and well-named
- Use meaningful variable names

## Pull Request Guidelines

1. **Description**: Provide a clear description of what your PR does
2. **Tests**: Include tests for new functionality
3. **Documentation**: Update documentation if you change APIs
4. **Commits**: Use clear, descriptive commit messages
5. **Checks**: Ensure all CI checks pass

## Common Issues

### ESP32 is not in the workspace
The `boid-esp32` crate is intentionally excluded from the workspace because it requires ESP Rust toolchain (Xtensa architecture for ESP32-S3). This is by design - build it separately when needed.

### Testing ESP32
The esp32 crate is excluded from default workspace members. To test it:
```bash
# Check it builds correctly
make test-esp32

# Or manually with ESP toolchain:
cd boid-esp32
cargo +esp check --target xtensa-esp32s3-none-elf

# Or run/flash to actual hardware:
cd boid-esp32
cargo run --release
```

**Prerequisites for ESP32-S3:**
- ESP Rust toolchain: `cargo install espup && espup install`
- Environment: `. $HOME/export-esp.sh` (run after espup install)
- espflash: `cargo install espflash`

**For ESP32-C3/C6 (RISC-V architecture):**
- Nightly toolchain: `rustup toolchain install nightly`
- RISC-V target: `rustup target add riscv32imc-unknown-none-elf --toolchain nightly`
- Update Cargo.toml features and .cargo/config.toml (see boid-esp32/README.md)

### WASM target not installed
```bash
rustup target add wasm32-unknown-unknown
```

## Useful Commands

```bash
# Format code
cargo fmt --all

# Check formatting without modifying files
cargo fmt --all -- --check

# Run clippy with warnings as errors
cargo clippy -p boid-core -p boid-wasm -- -D warnings

# Build WASM
cd boid-wasm
wasm-pack build --target web --out-dir www/pkg

# Run web server for testing
cd boid-wasm/www
python3 -m http.server 8080

# Check all targets
make check

# Clean build artifacts
cargo clean
```

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating a new one
- Be respectful and constructive in discussions

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
