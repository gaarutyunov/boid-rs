# Camera Support for XIAO ESP32S3 Sense

## Compatibility Challenge

The `esp32cam_rs` library requires `esp-idf-svc` which uses Rust `std` library, but our Embassy-based boid simulation uses `no_std` for better embedded performance.

## Solution Options

### Option 1: Hybrid Approach (Recommended for Testing)
Run camera in a separate binary that uses `std` + `esp-idf-svc`:

```
boid-embassy-std/     # New std-based camera server
├── Cargo.toml        # Uses esp-idf-svc, esp32cam
└── src/
    └── main.rs       # Camera MJPEG server only

boid-embassy/         # Existing no_std boid simulation
└── ... (unchanged)
```

Benefits:
- Camera streaming works immediately with `esp32cam_rs`
- Boid simulation stays lightweight with Embassy/no_std
- Can run both or separately

### Option 2: Manual FFI Bindings
Create no_std-compatible bindings to ESP-IDF camera driver.

Pros: Single binary, full control
Cons: Complex, requires unsafe code

### Option 3: Community no_std Camera Crate
Wait for or contribute to a no_std ESP32 camera library.

## Recommended Quick Start

For immediate testing, create a separate camera server:

```bash
# Create new std-based camera server
cargo new --bin boid-camera-server
cd boid-camera-server

# Add to Cargo.toml:
# esp32cam = "0.2"
# esp-idf-svc = "0.49"
# embassy-net (for HTTP)
```

Then run both on ESP32:
1. Camera server on one core (streams video)
2. Boid simulation on another core (receives position updates)

Or run camera server standalone and test with the boid-client.

## XIAO ESP32S3 Sense Pin Configuration

For reference when implementing camera support:

```rust
// Camera pins for XIAO ESP32S3 Sense (OV2640)
XCLK:  GPIO10
SIOD:  GPIO40  // I2C SDA
SIOC:  GPIO39  // I2C SCL
Y9:    GPIO48  // Data bit 7
Y8:    GPIO11  // Data bit 6
Y7:    GPIO12  // Data bit 5
Y6:    GPIO14  // Data bit 4
Y5:    GPIO16  // Data bit 3
Y4:    GPIO18  // Data bit 2
Y3:    GPIO17  // Data bit 1
Y2:    GPIO15  // Data bit 0
PCLK:  GPIO13  // Pixel clock
VSYNC: GPIO38  // Vertical sync
HREF:  GPIO47  // Horizontal reference
```

## Testing Without Camera

Use the client's fallback mode with a local webcam:

```bash
cd boid-client
cargo run --release -- --server http://ESP32_IP --video-source 0
```

This tests the full pipeline without needing ESP32 camera hardware.
