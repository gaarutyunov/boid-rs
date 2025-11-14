# Boid Simulation - Rust + WebAssembly + Embedded

A flocking behavior simulation implementing Craig Reynolds' Boid algorithm, built with Rust for multiple platforms:
- WebAssembly for web browsers
- Embedded systems (ESP32-S3 Sense, C3, C6) with Embassy framework

## Features

- **Pure Rust Implementation**: Core boid algorithm written in Rust with comprehensive tests
- **no_std Support**: Core algorithm works on embedded systems without standard library
- **WebAssembly Frontend**: Interactive canvas-based visualization running in the browser
- **Embedded Support**: Runs on ESP32-S3 Sense, C3, and C6 microcontrollers with LED displays
- **Embassy Framework**: Async runtime for efficient embedded execution
- **Touch Support**: Works on both desktop (mouse) and mobile (touch) devices
- **Real-time Controls**: Adjust simulation parameters on the fly
- **Automatic Deployment**: GitHub Actions workflow for continuous deployment to GitHub Pages

## Project Structure

This is a Rust workspace with multiple crates:

```
boid-rs/
├── boid-core/          # Core boid algorithm implementation (no_std compatible)
│   ├── src/
│   │   └── lib.rs      # Vector math, Boid, and Flock logic
│   └── Cargo.toml
├── boid-wasm/          # WebAssembly frontend
│   ├── src/
│   │   └── lib.rs      # WASM bindings and canvas rendering
│   ├── www/            # Web assets
│   │   ├── index.html
│   │   ├── index.js
│   │   └── package.json
│   └── Cargo.toml
├── boid-embassy/       # ESP32-S3 Sense embedded implementation (also supports C3/C6)
│   ├── src/
│   │   ├── main.rs     # Main Embassy application
│   │   ├── display.rs  # ST7789 display driver wrapper
│   │   └── rng.rs      # Pseudo-random number generator
│   ├── .cargo/
│   │   └── config.toml # Build configuration
│   └── Cargo.toml
├── .github/
│   └── workflows/      # CI/CD workflows
│       ├── test.yml    # Testing workflow
│       └── deploy.yml  # GitHub Pages deployment
├── Cargo.toml          # Workspace configuration
└── README.md
```

## Boid Algorithm

The simulation implements three fundamental rules of flocking behavior:

1. **Separation**: Boids avoid crowding nearby flockmates
2. **Alignment**: Boids steer towards the average heading of nearby flockmates
3. **Cohesion**: Boids move toward the average position of nearby flockmates

Each rule can be individually weighted to create different flocking behaviors.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for building WASM)
- A modern web browser with WebAssembly support
- Python 3 (for local development server)

## Building and Running

### Testing the Core Algorithm

```bash
# Run all tests
cargo test --workspace

# Test only the core boid algorithm
cargo test -p boid-core

# Run tests with output
cargo test -- --nocapture
```

### Running E2E Tests

The project includes Playwright end-to-end tests for the web interface:

```bash
# Navigate to the web frontend
cd boid-wasm/www

# Install dependencies (first time only)
npm install

# Install Playwright browsers (first time only)
npx playwright install --with-deps chromium

# Build WASM first
cd ..
wasm-pack build --target web
cd www

# Run e2e tests
npm run test:e2e

# Run e2e tests in UI mode (interactive)
npm run test:e2e:ui
```

The e2e tests verify:
- Canvas initialization and boid rendering
- Pointer tracking (mouse and touch events)
- Seek behavior when pointer is pressed
- Boundary containment
- Mouse leave handling

### Building the WASM Application

```bash
# Navigate to the WASM crate
cd boid-wasm

# Build the WASM module
wasm-pack build --target web --out-dir www/pkg

# Start a local server
cd www
python3 -m http.server 8080

# Open http://localhost:8080 in your browser
```

### Running Code Quality Checks

```bash
# Check formatting
cargo fmt --all -- --check

# Run clippy linter
cargo clippy --all-targets --all-features -- -D warnings
```

## Usage

### Interactive Controls

Once the application is running in your browser:

- **Add Boids**: Click or tap anywhere on the canvas to add new boids
- **Adjust Parameters**: Use the sliders to modify:
  - Separation Weight (0-3)
  - Alignment Weight (0-3)
  - Cohesion Weight (0-3)
  - Max Speed (1-10)
  - Max Force (0.01-0.5)

### Embedded (ESP32-S3 Sense)

For running on Xiao ESP32-S3 Sense (default) with an LED display:

```bash
# Navigate to the embassy crate
cd boid-embassy

# Build and flash to ESP32-S3
cargo run --release
```

See [boid-embassy/README.md](boid-embassy/README.md) for detailed hardware setup and configuration.

### Using as a Library

You can use the core boid algorithm in your own Rust projects:

**For std environments (PC, web, etc.):**
```rust
use boid_core::{FlockStd, BoidConfig};

fn main() {
    // Create a flock with default configuration
    let mut flock = FlockStd::new(800.0, 600.0, 100);

    // Or with custom configuration
    let config = BoidConfig {
        max_speed: 5.0,
        max_force: 0.15,
        separation_distance: 30.0,
        alignment_distance: 60.0,
        cohesion_distance: 60.0,
        separation_weight: 2.0,
        alignment_weight: 1.2,
        cohesion_weight: 1.0,
    };
    let mut custom_flock = FlockStd::new_with_config(800.0, 600.0, 50, config);

    // Update the simulation
    loop {
        flock.update();
        // Render or process boids...
    }
}
```

**For no_std environments (embedded systems):**
```rust
#![no_std]
use boid_core::{Flock, Boid, BoidConfig, Vector2D};

fn main() {
    let config = BoidConfig::default();
    let mut flock = Flock::<32>::new(240.0, 240.0, config);

    // Add boids manually
    let boid = Boid::new(
        Vector2D::new(120.0, 120.0),
        Vector2D::new(1.0, 0.5)
    );
    flock.add_boid(boid).unwrap();

    // Update loop
    loop {
        flock.update();
    }
}
```

## GitHub Actions

This project includes two automated workflows:

### Test Workflow
Runs on every push and pull request to main/master:
- Runs all unit tests
- Checks code formatting
- Runs Clippy linter
- Builds WASM to ensure it compiles

### Deploy Workflow
Automatically deploys to GitHub Pages on push to main/master:
- Builds the WASM module
- Uploads artifacts
- Deploys to GitHub Pages

To enable GitHub Pages:
1. Go to repository Settings > Pages
2. Set Source to "GitHub Actions"
3. The site will be available at `https://<username>.github.io/<repo-name>/`

## Performance Considerations

- The algorithm is O(n²) for each update, where n is the number of boids
- For best performance, keep the number of boids under 200 on most devices
- The WASM compilation provides near-native performance in the browser
- Touch events are debounced to prevent adding too many boids at once

## Development

### Quick Start

Before committing any changes, run:
```bash
make check
```

This runs all tests, linters, and format checks. See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

### Testing and Quality Checks

**Run tests:**
```bash
# Test all default packages (core + WASM)
make test

# Or run all workspace tests:
cargo test --workspace

# Check embassy builds (requires ESP toolchain):
make test-embassy
```

**Run linter:**
```bash
make clippy

# Or manually:
cargo clippy -p boid-core -p boid-wasm -- -D warnings
```

**Format code:**
```bash
make fmt

# Or check formatting:
make fmt-check
```

**Run all checks (recommended before committing):**
```bash
make check
# Or with embassy:
make check-embassy
```

**Note:** The `boid-embassy` crate is **excluded from the workspace** entirely since it requires ESP Rust toolchain (Xtensa architecture for ESP32-S3). Build it separately: `cd boid-embassy && cargo build`, or use `make test-embassy` to check it builds correctly. For C3/C6 support, see boid-embassy/README.md.

### Adding New Features

1. **Core Algorithm Changes**: Modify `boid-core/src/lib.rs`
2. **WASM Bindings**: Update `boid-wasm/src/lib.rs`
3. **UI Changes**: Edit `boid-wasm/www/index.html` and `index.js`
4. **Embedded Changes**: Update `boid-embassy/src/`

### Writing Tests

Always add tests for new features:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        // Your test here
    }
}
```

Run tests before committing:
```bash
make check
# or
cargo test --workspace
```

## License

MIT

## References

- [Craig Reynolds' Boids](http://www.red3d.com/cwr/boids/)
- [WebAssembly](https://webassembly.org/)
- [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Setting up your development environment
- Running tests and checks before committing
- Code style and best practices
- Creating pull requests
