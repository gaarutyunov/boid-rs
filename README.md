# Boid Simulation - Rust + WebAssembly

A flocking behavior simulation implementing Craig Reynolds' Boid algorithm, built with Rust and compiled to WebAssembly for web deployment.

## Features

- **Pure Rust Implementation**: Core boid algorithm written in Rust with comprehensive tests
- **WebAssembly Frontend**: Interactive canvas-based visualization running in the browser
- **Touch Support**: Works on both desktop (mouse) and mobile (touch) devices
- **Real-time Controls**: Adjust simulation parameters on the fly
- **Automatic Deployment**: GitHub Actions workflow for continuous deployment to GitHub Pages

## Project Structure

This is a Rust workspace with multiple crates:

```
boid-rs/
├── boid-core/          # Core boid algorithm implementation
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

### Using as a Library

You can use the core boid algorithm in your own Rust projects:

```rust
use boid_core::{Flock, BoidConfig};

fn main() {
    // Create a flock with default configuration
    let mut flock = Flock::new(800.0, 600.0, 100);

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
    let mut custom_flock = Flock::new_with_config(800.0, 600.0, 50, config);

    // Update the simulation
    loop {
        flock.update();
        // Render or process boids...
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

### Adding New Features

1. **Core Algorithm Changes**: Modify `boid-core/src/lib.rs`
2. **WASM Bindings**: Update `boid-wasm/src/lib.rs`
3. **UI Changes**: Edit `boid-wasm/www/index.html` and `index.js`

### Testing

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

## License

MIT

## References

- [Craig Reynolds' Boids](http://www.red3d.com/cwr/boids/)
- [WebAssembly](https://webassembly.org/)
- [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
