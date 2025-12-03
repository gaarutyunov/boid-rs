# Claude Agent Instructions

This document provides guidance for AI agents (like Claude) working on this project.

## Project Overview

This is a multi-platform Rust boid simulation with five main components:
- **boid-core**: Pure Rust no_std-compatible algorithm
- **boid-shared**: Shared types for client-server communication (no_std compatible)
- **boid-wasm**: WebAssembly frontend for browsers with MediaPipe hand tracking
- **boid-esp32**: Embedded ESP32 implementation with WiFi and HTTP server
- **boid-client**: Desktop/Raspberry Pi client with OpenCV hand tracking

## Architecture

### Core Algorithm (`boid-core/src/lib.rs`)
- `Vector2D`: 2D vector math (lines 7-114)
- `Boid`: Individual boid entity with position, velocity, acceleration (lines 124-192)
- `BoidConfig`: Simulation parameters (lines 195-199)
- `behavior` module: Flocking behaviors - separation, alignment, cohesion, seek (lines 211-302)
- `Flock<N>`: Fixed-size flock for embedded (lines 305-352)
- `FlockStd`: Dynamic-size flock for std environments (lines 356-448)

### WASM Interface (`boid-wasm/src/lib.rs`)
- `BoidSimulation`: Main struct exposing methods to JavaScript
- Canvas rendering using web-sys
- Pointer tracking for interactive behavior
- Methods: `new()`, `update()`, `render()`, event handlers

### Web Frontend (`boid-wasm/www/`)
- `index.html`: Main HTML with canvas and controls
- `index.js`: JavaScript glue code, animation loop, event handlers, MediaPipe integration
- `package.json`: npm scripts for building and testing
- `tests/`: Playwright e2e tests

### Shared Types (`boid-shared/src/lib.rs`)
- `Position`: 2D position with distance calculations
- `HandLandmarks`: Thumb and index finger positions with pinch distance
- `TargetPositionUpdate`: API type for updating boid target
- `BoidSettings`: Configuration parameters
- `SettingsUpdate`: API type for updating settings
- `StatusResponse`: Server status information
- All types use serde for JSON serialization (optional std feature)

### ESP32 (`boid-esp32/src/`)
- `main.rs`: Main ESP32 application with WiFi and HTTP server
- `http_server.rs`: HTTP API handlers for position and settings endpoints
- `wifi_config.rs`: WiFi credentials (loaded from environment variables)
- `display.rs`: ST7789 display driver wrapper
- `rng.rs`: Pseudo-random number generator

### Client (`boid-client/src/`)
- `main.rs`: CLI application, camera capture, HTTP client, visualization
- `hand_tracker.rs`: OpenCV-based hand detection using skin color and contour analysis

## Development Workflow

### Before Making Changes
1. Read relevant code sections to understand the context
2. Check tests to understand expected behavior
3. Verify the change won't break existing functionality

### Testing Strategy
1. **Unit tests**: `cargo test --workspace` (for Rust code)
2. **E2E tests**: `npm run test:e2e` from `boid-wasm/www/` (for web interface)
3. **Build verification**: `wasm-pack build --target web` in `boid-wasm/`

### Code Quality & Pre-commit Hooks

The project uses formatting and linting tools to maintain code quality:

**Formatting:**
- Run `cargo fmt --all` to format all Rust code
- Check formatting with `cargo fmt --all -- --check`

**Linting:**
- Run `cargo clippy --all-targets --all-features -- -D warnings`
- Fix clippy suggestions before committing

**Setting up pre-commit hooks:**
You can set up Git hooks to automatically run formatting and linting before each commit:

```bash
# Create .git/hooks/pre-commit
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash

echo "Running pre-commit checks..."

# Format check
echo "Checking code formatting..."
cargo fmt --all -- --check
if [ $? -ne 0 ]; then
    echo "❌ Code formatting check failed. Run 'cargo fmt --all' to fix."
    exit 1
fi

# Clippy check
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "❌ Clippy check failed. Fix the warnings above."
    exit 1
fi

echo "✅ Pre-commit checks passed!"
EOF

# Make the hook executable
chmod +x .git/hooks/pre-commit
```

Alternatively, auto-format on commit (less strict):
```bash
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
cargo fmt --all
git add -u
EOF

chmod +x .git/hooks/pre-commit
```

### Common Tasks

#### Adding New Boid Behavior
1. Add behavior function in `boid-core/src/lib.rs` `behavior` module
2. Update `FlockStd::update_with_target()` to apply the behavior
3. Add tests in the `tests` module at the bottom of the file
4. Run `cargo test -p boid-core` to verify

#### Modifying WASM Interface
1. Update `boid-wasm/src/lib.rs` with new methods
2. Mark public methods with `#[wasm_bindgen]`
3. Rebuild with `wasm-pack build --target web`
4. Update JavaScript in `boid-wasm/www/index.js` to use new methods
5. Add e2e tests in `boid-wasm/www/tests/` if UI behavior changes

#### Updating Web UI
1. Modify `boid-wasm/www/index.html` for structure/controls
2. Update `boid-wasm/www/index.js` for behavior/event handlers
3. Add/update e2e tests to verify changes
4. Test locally with `python3 -m http.server 8080` in `boid-wasm/www/`

### Build Commands

```bash
# Test all Rust code
cargo test --workspace

# Test WASM bindings (requires wasm-pack and a browser)
cd boid-wasm && wasm-pack test --headless --chrome

# Build WASM
cd boid-wasm && wasm-pack build --target web --out-dir www/pkg

# Run e2e tests (requires WASM to be built first)
cd boid-wasm/www
npm install              # First time only
npm run test:e2e        # Run tests
npm run test:e2e:ui     # Run with interactive UI

# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
```

## Key Patterns

### Coordinate System
- Canvas coordinates: (0,0) is top-left
- Boids use canvas coordinate system
- Boundary: `contain_within_bounds()` bounces off edges

### Pointer Tracking
- State stored in `BoidSimulation`: `pointer_position`, `pointer_pressed`
- When pressed: boids seek the pointer position (2x weight)
- When released: boids fly freely within bounds
- JavaScript events → Rust handlers → simulation update

### Update Flow
1. JavaScript animation loop calls `simulation.update()`
2. Rust determines target (pointer position if pressed, None otherwise)
3. `FlockStd::update_with_target(target)` calculates forces
4. Apply forces: separation, alignment, cohesion, seek (if target exists)
5. Update each boid: velocity, position
6. Apply boundary containment

## Testing Guidelines

### WASM Unit Tests
Tests are in `boid-wasm/src/lib.rs` using `wasm-bindgen-test`:
- Run with `wasm-pack test --headless --chrome` (requires Chrome/Chromium)
- Tests run in a headless browser environment (not in standard `cargo test`)
- Test pointer tracking state management, configuration updates, and simulation behavior
- Create test canvas elements dynamically for each test
- Use `#[wasm_bindgen_test]` attribute instead of `#[test]`

### E2E Test Structure
Tests are in `boid-wasm/www/tests/pointer-tracking.spec.js`:
- Use `page.evaluate()` to access `window.simulation`
- Wait for initialization with `waitForFunction()`
- Simulate events with `page.mouse.*` and canvas locators
- Verify behavior by checking boid positions or console logs

### Adding E2E Tests
```javascript
test('should do something', async ({ page }) => {
  await page.goto('/');
  await page.waitForFunction(() => window.simulation !== undefined);

  const canvas = await page.locator('#canvas');
  // ... test interactions

  // Verify results
  const result = await page.evaluate(() => {
    // Access window.simulation here
  });
  expect(result).toBe(expectedValue);
});
```

## CI/CD

GitHub Actions workflows in `.github/workflows/`:
- `ci.yml`: Runs tests, builds WASM, runs e2e tests
- Caching: npm dependencies, Playwright browsers, cargo artifacts
- E2E tests run on every push/PR

## Common Pitfalls

1. **WASM not rebuilt**: Always rebuild WASM after Rust changes before testing web UI
2. **Coordinate conversion**: Remember to convert between screen and canvas coordinates
3. **Async timing**: E2E tests may need `waitForTimeout()` for animation to complete
4. **no_std compatibility**: Core algorithm must work without std (no `Vec`, use `heapless::Vec`)
5. **wasm-opt disabled**: The project disables wasm-opt for compatibility (see `Cargo.toml`)

## Client-Server Architecture

The ESP32 streams camera to the client, which processes frames and sends control commands back:

```
ESP32-S3 Sense          →  WiFi  →         PC/Raspberry Pi
──────────────                             ────────────────
Camera Module                              OpenCV Processing
     ↓                                          ↓
MJPEG Stream         →  HTTP GET   →      VideoCapture
     ↓                                          ↓
Boid Simulation      ←  HTTP POST   ←      Hand Tracking
     ↓                                          ↓
LCD Display                                Position Updates
```

### ESP32 Side (Server)
1. WiFi credentials loaded from environment variables (`WIFI_SSID`, `WIFI_PASSWORD`)
2. HTTP server listens on port 80
3. Endpoints:
   - `GET /stream` - Stream camera as MJPEG (requires camera driver implementation)
   - `POST /api/position` - Update target position
   - `POST /api/settings` - Update boid configuration
   - `GET /api/status` - Get simulation status
4. Camera module (OV2640) connected via I2C and parallel interface
5. Updates sent via channels to main simulation loop
6. Main loop checks channels non-blockingly each frame

**Note**: Camera streaming has a std/no_std compatibility issue:
- esp32cam_rs (recommended) requires std via esp-idf-svc
- See `boid-esp32/README_CAMERA.md` for hybrid approach options
- See `boid-esp32/src/camera.rs` for pin config and reference code
- Test without ESP32 camera: use client with `--video-source 0`

### Client Side
1. Opens video stream from ESP32 using OpenCV VideoCapture
   - Default: `http://ESP32_IP/stream` (MJPEG stream from ESP32 camera)
   - Fallback: Local camera device ID (e.g., 0, 1, 2...)
2. Processes each frame for hand detection:
   - Convert to HSV color space
   - Detect skin color regions
   - Find contours
   - Extract finger tip positions from largest contour
3. Sends HTTP POST requests when position changes significantly (>5px)
4. Displays visualization window with hand tracking overlay

## File Locations

- Core algorithm: `boid-core/src/lib.rs`
- Shared types: `boid-shared/src/lib.rs`
- WASM bindings: `boid-wasm/src/lib.rs`
- Web UI HTML: `boid-wasm/www/index.html`
- Web UI JS: `boid-wasm/www/index.js`
- E2E tests: `boid-wasm/www/tests/*.spec.js`
- ESP32 main: `boid-esp32/src/main.rs`
- HTTP server: `boid-esp32/src/http_server.rs`
- Camera module (reference): `boid-esp32/src/camera.rs`
- Camera documentation: `boid-esp32/README_CAMERA.md`
- WiFi config: `boid-esp32/src/wifi_config.rs`
- Client main: `boid-client/src/main.rs`
- Hand tracker: `boid-client/src/hand_tracker.rs`
- CI config: `.github/workflows/ci.yml`
- Playwright config: `boid-wasm/www/playwright.config.js`

## Helpful Commands

```bash
# Quick check before committing
cargo test --workspace && cargo fmt --all -- --check && cargo clippy --all-targets -- -D warnings

# Build and test WASM
cd boid-wasm
wasm-pack build --target web
cd www
npm run test:e2e

# View e2e test results
npm run test:e2e:ui  # Interactive mode

# Test ESP32 build (requires ESP toolchain)
cd boid-esp32
cargo +esp check

# Build client (requires OpenCV)
cd boid-client
cargo build --release

# Run client
cargo run --release -- --server http://192.168.1.100

# Test shared crate
cargo test -p boid-shared
```

## When in Doubt

1. Check existing tests for patterns
2. Read the referenced source file sections
3. Verify changes don't break existing tests
4. Add tests for new functionality
5. Update documentation if behavior changes
