# QEMU Support for ESP32-S3 Boid Simulation

This document describes how to build and test the ESP32-S3 boid simulation using QEMU emulation.

## Overview

QEMU support allows testing the boid algorithm and core functionality without physical ESP32 hardware. Two modes are available:

1. **Headless Mode** (`qemu` feature): Basic boid algorithm testing without display or camera (for CI/CD)
2. **Display Mode** (`qemu-display` feature): Local development with emulated display (not yet implemented)

## Prerequisites

### Install ESP32-S3 QEMU

You need a QEMU build with ESP32-S3 support. Follow the instructions at:
https://github.com/espressif/esp-toolchain-docs/blob/main/qemu/esp32s3/README.md

Quick install steps:
```bash
# Clone QEMU with ESP32-S3 support
git clone https://github.com/espressif/qemu.git
cd qemu
git checkout esp-develop

# Install dependencies (Ubuntu/Debian)
sudo apt-get install libgcrypt-dev libslirp-dev libsdl2-dev

# Configure and build
./configure \
    --target-list=xtensa-softmmu \
    --enable-gcrypt \
    --enable-slirp \
    --enable-debug \
    --enable-sdl \
    --disable-strip \
    --disable-user \
    --disable-capstone \
    --disable-vnc \
    --disable-gtk

ninja -C build

# Set environment variable
export ESP_QEMU_PATH=/path/to/qemu/build/qemu-system-xtensa
```

### Install ESP Rust Toolchain

```bash
# Install espup
cargo install espup

# Install ESP Rust toolchain
espup install

# Source the environment
source ~/export-esp.sh
```

## Building for QEMU

### Using the Run Script (Recommended)

```bash
cd boid-esp32
./run_qemu.sh
```

The script will:
1. Build the project with `--features qemu`
2. Create a flash image
3. Run QEMU with appropriate settings
4. Timeout after 60 seconds (configurable)

### Manual Build

```bash
# Set QEMU-specific SDK config
export ESP_IDF_SDKCONFIG_DEFAULTS="sdkconfig.qemu"

# Build with QEMU feature
cargo +esp build --release --features qemu

# Create flash image
cargo +esp espflash save-image \
    --chip esp32s3 \
    --release \
    --features qemu \
    target/xtensa-esp32s3-espidf/release/boid-esp32 \
    target/flash_image.bin

# Run QEMU
qemu-system-xtensa \
    -nographic \
    -machine esp32s3 \
    -drive file=target/flash_image.bin,if=mtd,format=raw \
    -nic user,model=open_eth \
    -global driver=timer.esp32s3.timg,property=wdt_disable,value=true
```

## What Gets Tested in QEMU

### Headless Mode (`qemu` feature)

The headless mode tests:
- ✅ WiFi initialization and connection
- ✅ HTTP server startup
- ✅ Boid algorithm (900 iterations, ~30 seconds)
- ✅ Position updates via API
- ✅ Settings updates via API
- ✅ Status endpoint
- ❌ Camera streaming (returns 501 Not Implemented)
- ❌ Display rendering (skipped)

The simulation runs for 900 iterations (30 seconds at 33ms per iteration) and logs progress:
```
I (1234) boid-esp32: Iteration 0/900: Boid[0] at (120.50, 85.30), vel (1.25, -0.87)
I (11234) boid-esp32: Iteration 300/900: Boid[0] at (145.20, 92.15), vel (0.98, 1.42)
I (21234) boid-esp32: Iteration 600/900: Boid[0] at (110.75, 120.40), vel (-1.15, 0.65)
I (31234) boid-esp32: QEMU test completed successfully!
```

### Display Mode (`qemu-display` feature)

*Not yet implemented* - Will support SDL-based display emulation for local development.

## CI/CD Integration

The GitHub Actions workflow runs QEMU tests automatically:

```yaml
qemu-test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install ESP QEMU
      run: |
        # Install pre-built QEMU or build from source
    - name: Install ESP Rust toolchain
      run: |
        cargo install espup
        espup install
    - name: Run QEMU tests
      run: |
        cd boid-esp32
        ./run_qemu.sh
```

## Configuration

### QEMU-Specific SDK Settings (`sdkconfig.qemu`)

```
# Enable OpenCores Ethernet for networking
CONFIG_ETH_USE_OPENETH=y

# Disable hardware crypto (not available in QEMU)
CONFIG_MBEDTLS_HARDWARE_AES=n
CONFIG_MBEDTLS_HARDWARE_SHA=n
CONFIG_MBEDTLS_HARDWARE_MPI=n
CONFIG_MBEDTLS_HARDWARE_ECC=n

# Disable watchdogs
CONFIG_ESP_TASK_WDT_INIT=n
CONFIG_ESP_INT_WDT=n
```

### Environment Variables

- `ESP_QEMU_PATH`: Path to qemu-system-xtensa binary (default: `qemu-system-xtensa`)
- `FLASH_SIZE`: Flash size for image creation (default: `4MB`)
- `QEMU_TIMEOUT`: Test timeout in seconds (default: `60`)
- `ESP_IDF_SDKCONFIG_DEFAULTS`: SDK config file (set to `sdkconfig.qemu` for QEMU builds)

## Troubleshooting

### "qemu-system-xtensa: command not found"

Set the `ESP_QEMU_PATH` environment variable:
```bash
export ESP_QEMU_PATH=/path/to/qemu/build/qemu-system-xtensa
```

### Build fails with "CONFIG_ETH_USE_OPENETH not found"

Make sure you're using the QEMU SDK config:
```bash
export ESP_IDF_SDKCONFIG_DEFAULTS="sdkconfig.qemu"
```

### QEMU crashes or hangs

1. Check QEMU version - make sure you're using the ESP fork with ESP32-S3 support
2. Verify watchdog is disabled in QEMU command line
3. Check QEMU logs for errors

### WiFi doesn't connect in QEMU

QEMU uses emulated networking. The WiFi credentials in `wifi_config.rs` don't matter for QEMU - it will automatically provide a virtual network interface.

## Limitations

- Camera streaming is not available (QEMU doesn't emulate the camera module)
- Display output is not visible in headless mode
- Some hardware peripherals may not be fully emulated
- Performance may be slower than real hardware
- RTC watchdog timer emulation is incomplete

## Future Enhancements

- [ ] Implement `qemu-display` feature with SDL support
- [ ] Add automated API endpoint testing in QEMU
- [ ] Support for display screenshots in CI/CD
- [ ] Network traffic validation
- [ ] Performance benchmarking in QEMU
