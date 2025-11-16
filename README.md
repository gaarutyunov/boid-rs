# Boid Simulation - Rust + WebAssembly + Embedded + Hand Tracking

A flocking behavior simulation implementing Craig Reynolds' Boid algorithm, built with Rust for multiple platforms:
- WebAssembly for web browsers with MediaPipe hand tracking
- Embedded systems (ESP32-S3 Sense, C3, C6) with WiFi HTTP server
- Desktop/Raspberry Pi client with OpenCV hand tracking for controlling remote ESP32 devices

## Features

- **Pure Rust Implementation**: Core boid algorithm written in Rust with comprehensive tests
- **no_std Support**: Core algorithm works on embedded systems without standard library
- **WebAssembly Frontend**: Interactive canvas-based visualization running in the browser
- **Hand Gesture Control**: Control boids with hand pinch gestures using webcam
  - **WASM**: MediaPipe hand tracking in the browser
  - **Client**: OpenCV-based hand tracking for desktop/Raspberry Pi
- **Embedded Support**: Runs on ESP32-S3 Sense, C3, and C6 microcontrollers with LED displays
- **WiFi HTTP Server**: ESP32 hosts HTTP API for remote control
- **Client-Server Architecture**: Control ESP32 boids from PC/Raspberry Pi via WiFi
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
├── boid-shared/        # Shared types for client-server communication
│   ├── src/
│   │   └── lib.rs      # Position, HandLandmarks, API types
│   └── Cargo.toml
├── boid-wasm/          # WebAssembly frontend with MediaPipe hand tracking
│   ├── src/
│   │   └── lib.rs      # WASM bindings and canvas rendering
│   ├── www/            # Web assets
│   │   ├── index.html
│   │   ├── index.js    # MediaPipe integration
│   │   └── package.json
│   └── Cargo.toml
├── boid-esp32/         # ESP32-S3 embedded impl with WiFi HTTP server
│   ├── src/
│   │   ├── main.rs     # Main ESP32 application
│   │   ├── http_server.rs  # HTTP API server
│   │   ├── wifi_config.rs  # WiFi credentials
│   │   ├── display.rs  # ST7789 display driver wrapper
│   │   └── rng.rs      # Pseudo-random number generator
│   ├── .cargo/
│   │   └── config.toml # Build configuration
│   ├── .env.example    # WiFi configuration template
│   └── Cargo.toml
├── boid-client/        # Desktop/Raspberry Pi client with OpenCV
│   ├── src/
│   │   ├── main.rs     # Client application and CLI
│   │   └── hand_tracker.rs  # OpenCV hand tracking
│   └── Cargo.toml
├── .github/
│   └── workflows/      # CI/CD workflows
│       ├── test.yml    # Testing workflow
│       └── deploy.yml  # GitHub Pages deployment
├── Cargo.toml          # Workspace configuration
├── CLAUDE.md           # Development guidelines
└── README.md
```

## Architecture Modes

### 1. Web Mode (WASM + MediaPipe)
Runs entirely in the browser with hand gesture control:
```
Browser → MediaPipe Hand Tracking → WASM Boid Simulation → Canvas Display
```

### 2. Embedded Mode (ESP32 Only)
Standalone ESP32 with display showing autonomous boids:
```
ESP32 → Boid Simulation → LCD Display
```

### 3. Client-Server Mode (ESP32 Camera + OpenCV Processing)
ESP32 streams camera to PC/Raspberry Pi for processing:
```
┌──────────────────┐         ┌─────────────────────┐
│   ESP32-S3       │         │  PC/Raspberry Pi    │
│                  │         │                     │
│  Camera Module   │  WiFi   │  OpenCV Processing  │
│      ↓           │────────>│       ↓             │
│  MJPEG Stream    │  HTTP   │  Hand Tracking      │
│      ↓           │  GET    │       ↓             │
│  Boid Simulation │<────────│  Position Updates   │
│      ↓           │  POST   │                     │
│  LCD Display     │         │                     │
└──────────────────┘         └─────────────────────┘
```

Benefits of client-server mode:
- **Offload processing**: Heavy OpenCV hand tracking runs on powerful PC/RPi
- **No client camera needed**: Uses ESP32's built-in camera module
- **Remote processing**: Process video from ESP32 anywhere on the network
- **Scalability**: One powerful machine can process streams from multiple ESP32 devices

## Boid Algorithm

The simulation implements three fundamental rules of flocking behavior:

1. **Separation**: Boids avoid crowding nearby flockmates
2. **Alignment**: Boids steer towards the average heading of nearby flockmates
3. **Cohesion**: Boids move toward the average position of nearby flockmates

Each rule can be individually weighted to create different flocking behaviors.

## Prerequisites

### Common Requirements
- [Rust](https://www.rust-lang.org/tools/install) (1.70 or later)

### For Web Version (WASM)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A modern web browser with WebAssembly support
- Python 3 (for local development server)
- Webcam (for hand tracking)

### For ESP32 Embedded Version
- Rust nightly toolchain
- ESP32-S3 Sense or compatible board
- SPI LCD display (ST7789 or compatible, 240x240)
- [espflash](https://github.com/esp-rs/espflash) for flashing

### For Client-Server Mode
**ESP32 Requirements:**
- All embedded requirements above
- WiFi network

**Client Requirements (PC/Raspberry Pi):**
- OpenCV development libraries
- Clang/LLVM (for OpenCV Rust bindings)
- Webcam or USB camera
- Network connectivity to ESP32

#### Installing OpenCV

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install libopencv-dev clang libclang-dev
```

**macOS:**
```bash
brew install opencv
```

**Raspberry Pi:**
```bash
sudo apt-get update
sudo apt-get install libopencv-dev clang libclang-dev
```

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
# Navigate to the esp32 crate
cd boid-esp32

# Build and flash to ESP32-S3
cargo run --release
```

See [boid-esp32/README.md](boid-esp32/README.md) for detailed hardware setup and configuration.

### Client-Server Mode (ESP32 Camera Stream + Hand Tracking)

ESP32 streams its camera to PC/Raspberry Pi for hand tracking processing.

#### Step 1: Configure and Flash ESP32

1. Create WiFi configuration file:
```bash
cd boid-esp32
cp .env.example .env
```

2. Edit `.env` with your WiFi credentials:
```bash
WIFI_SSID=YourNetworkName
WIFI_PASSWORD=YourPassword
```

3. Build and flash ESP32-S3 Sense:
```bash
cargo +nightly run --release
```

**Note**: Camera streaming requires additional implementation. See `boid-esp32/src/camera.rs` for details.

4. Note the IP address displayed on the serial console or LCD (e.g., `192.168.1.100`)

#### Step 2: Run Client on PC/Raspberry Pi

1. Build and run the client (streaming from ESP32 camera):
```bash
cd boid-client
cargo run --release -- --server http://192.168.1.100
```
(Replace with your ESP32's IP address)

2. The client will:
   - Connect to ESP32 camera stream at `http://192.168.1.100/stream`
   - Process each frame for hand tracking using OpenCV
   - Detect hand and finger positions
   - Send position updates back to ESP32 over WiFi

3. Control the boids:
   - Show your hand to the ESP32's camera
   - Move your index finger to set the target position
   - Boids on the ESP32 display will follow your finger!

#### Client Command-Line Options

```bash
# Stream from ESP32 camera (default)
boid-client --server http://192.168.1.100

# Use local camera for testing (fallback mode)
boid-client --server http://192.168.1.100 --video-source 0

# Hide the preview window (for headless operation)
boid-client --server http://192.168.1.100 --show-window false

# Enable debug logging
boid-client --server http://192.168.1.100 --debug

# Press 'q' in the preview window to quit
```

#### Implementation Status

⚠️ **Camera Streaming Compatibility:**

The recommended ESP32 camera library (`esp32cam_rs`) requires `std` via `esp-idf-svc`.

**Current Options:**
1. **Hybrid Approach**: Run separate std-based camera server alongside boid simulation (see `boid-esp32/README_CAMERA.md`)
2. **Manual FFI**: Create custom bindings to ESP-IDF camera driver (complex)
3. **Client Fallback**: Test full pipeline using client's local camera with `--video-source 0`

**Pin Configuration and Reference Implementation:**
- See `boid-esp32/src/camera.rs` for XIAO ESP32S3 Sense pin mapping
- Reference `esp32cam_rs` implementation provided in comments
- Link to working webserver example: https://github.com/Kezii/esp32cam_rs

**Quick Test Without ESP32 Camera:**
```bash
cd boid-client
cargo run --release -- --server http://ESP32_IP --video-source 0
```

This tests the complete hand tracking → ESP32 control pipeline using your computer's webcam.

### HTTP API (ESP32)

The ESP32 exposes a REST API for remote control:

#### GET /stream
Stream camera feed as MJPEG (requires camera implementation):
```bash
# View in browser
open http://192.168.1.100/stream

# Or use with OpenCV (automatic in boid-client)
```

**Note**: Camera streaming endpoint requires ESP-IDF camera driver integration.
See `boid-esp32/src/camera.rs` for implementation details.

#### POST /api/position
Set target position for boids to seek:
```bash
curl -X POST http://192.168.1.100/api/position \
  -H "Content-Type: application/json" \
  -d '{"position":{"x":120.0,"y":120.0}}'
```

Clear target (free flying):
```bash
curl -X POST http://192.168.1.100/api/position \
  -H "Content-Type: application/json" \
  -d '{"position":null}'
```

#### POST /api/settings
Update simulation parameters:
```bash
curl -X POST http://192.168.1.100/api/settings \
  -H "Content-Type: application/json" \
  -d '{
    "settings": {
      "separation_weight": 1.5,
      "alignment_weight": 1.0,
      "cohesion_weight": 1.0,
      "max_speed": 2.0,
      "max_force": 0.05,
      "seek_weight": 8.0
    }
  }'
```

#### GET /api/status
Get current simulation status:
```bash
curl http://192.168.1.100/api/status
```
Response:
```json
{
  "boid_count": 20,
  "fps": 30,
  "target_active": true
}
```

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
# Or with esp32:
make check-esp32
```

**Note:** The `boid-esp32` crate is **excluded from the workspace** entirely since it requires ESP Rust toolchain (Xtensa architecture for ESP32-S3). Build it separately: `cd boid-esp32 && cargo build`, or use `make test-esp32` to check it builds correctly. For C3/C6 support, see boid-esp32/README.md.

### Adding New Features

1. **Core Algorithm Changes**: Modify `boid-core/src/lib.rs`
2. **WASM Bindings**: Update `boid-wasm/src/lib.rs`
3. **UI Changes**: Edit `boid-wasm/www/index.html` and `index.js`
4. **Embedded Changes**: Update `boid-esp32/src/`

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
