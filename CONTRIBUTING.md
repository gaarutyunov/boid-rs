# Contributing to Boid Simulation

Thank you for your interest in contributing to this project! This guide will help you set up your development environment and understand the contribution workflow.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for WASM development)
- For embedded development:
  - Rust nightly: `rustup toolchain install nightly`
  - RISC-V target: `rustup target add riscv32imc-unknown-none-elf --toolchain nightly`
  - espflash: `cargo install espflash`

## Development Workflow

### Before Committing

**IMPORTANT:** Always run the following checks before committing code:

```bash
# Quick check - runs all standard environment checks
make check

# Or manually run each check:
cargo test -p boid-core -p boid-wasm    # Run tests
cargo clippy -p boid-core -p boid-wasm -- -D warnings  # Run linter
cargo fmt --all -- --check              # Check formatting
```

If any of these fail, fix the issues before committing.

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
   - `boid-embassy/` - Embedded system changes

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

### boid-embassy
Embedded implementation for ESP32-C3/C6.

**Key files:**
- `src/main.rs` - Main application
- `src/display.rs` - Display driver
- `src/rng.rs` - Random number generator

**Note:** This crate requires special build configuration and can't be tested in the standard environment. See [boid-embassy/README.md](boid-embassy/README.md) for details.

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

### "profiles for the non root package will be ignored"
This is a warning you can ignore. It occurs because `boid-embassy` has its own profile settings.

### Embassy crate won't build in tests
The embassy crate requires special embedded toolchain configuration. Always test specific packages:
```bash
cargo test -p boid-core -p boid-wasm
```
Instead of:
```bash
cargo test --workspace  # This will fail
```

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
