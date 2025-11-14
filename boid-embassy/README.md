# Boid Simulation for ESP32-S3 Sense with Embassy

This project implements a real-time boid flocking simulation on the Xiao Seed ESP32-S3 Sense microcontroller using the Embassy async framework and an ST7789-based LCD display.

## Hardware Requirements

- **Xiao ESP32-S3 Sense board** (default)
  - Also compatible with ESP32-C3 and ESP32-C6 (requires feature flags)
- ST7789 240x240 LCD display (or compatible SPI display)
- Connecting wires

## Pin Configuration

Default pin configuration for **Xiao ESP32-S3 Sense**:
- GPIO8: SPI Clock (SCK)
- GPIO9: SPI Data (MOSI/COPI)
- GPIO7: Chip Select (CS)
- GPIO4: Data/Command (DC)
- GPIO5: Reset (RST)

You can modify these pins in `src/main.rs` if your hardware uses different connections.

## Building

### Prerequisites

1. Install the ESP Rust toolchain:
```bash
# Install espup (ESP Rust installer)
cargo install espup

# Install ESP Rust toolchain
espup install

# Source the environment (add to your shell profile)
. $HOME/export-esp.sh
```

2. Install espflash for flashing:
```bash
cargo install espflash
```

### Build the project

```bash
cd boid-embassy
cargo build --release
```

## Flashing

Connect your ESP32-S3 Sense board via USB and run:

```bash
cargo run --release
```

Or manually flash:
```bash
espflash flash target/xtensa-esp32s3-none-elf/release/boid-embassy --monitor
```

## Configuration

### Display Size

The default display size is 240x240 pixels. Modify `DISPLAY_WIDTH` and `DISPLAY_HEIGHT` in `src/main.rs` if using a different display.

### Boid Parameters

Adjust the boid behavior in `src/main.rs`:
- `NUM_BOIDS`: Number of boids in the simulation (default: 20)
- `BOID_SIZE`: Visual size of each boid (default: 3 pixels)
- `BoidConfig`: Fine-tune flocking behavior parameters

### Frame Rate

The simulation targets ~30 FPS. Adjust the delay in the main loop if needed:
```rust
Timer::after(Duration::from_millis(33)).await; // ~30 FPS
```

## ESP32-C3/C6 Support

To build for ESP32-C3 or C6, update `boid-embassy/Cargo.toml`:

**For ESP32-C3:**
```toml
[dependencies]
embassy-executor = { version = "0.6", features = ["arch-riscv32", "executor-thread"] }
esp-hal = { version = "0.21", features = ["esp32c3"] }
esp-backtrace = { version = "0.14", features = ["esp32c3", "panic-handler", "exception-handler", "println"] }
esp-println = { version = "0.12", features = ["esp32c3", "log"] }

[features]
default = ["esp32c3"]
```

And update `.cargo/config.toml`:
```toml
[build]
target = "riscv32imc-unknown-none-elf"
```

And `rust-toolchain.toml`:
```toml
[toolchain]
channel = "nightly"
targets = ["riscv32imc-unknown-none-elf"]
```

**For ESP32-C6:** Same as C3 but replace "esp32c3" with "esp32c6".

## Architecture

- `main.rs`: Main application loop, Embassy executor, display initialization
- `display.rs`: Display driver wrapper for ST7789
- `rng.rs`: Simple pseudo-random number generator for embedded use
- `boid-core`: Core boid algorithm (no_std compatible)

## Troubleshooting

### Display not working
- Check pin connections match your configuration
- Verify your display is ST7789-based (or modify the display driver)
- Ensure power supply is adequate (USB power should be sufficient)

### Build errors
- Make sure ESP Rust toolchain is installed: `espup install`
- Source the environment: `. $HOME/export-esp.sh`
- Check the target is correct for your chip

### Flashing errors
- Install/update espflash: `cargo install espflash --force`
- Check USB connection and permissions (may need `sudo usermod -a -G dialout $USER`)
- Try holding the BOOT button while connecting
- Verify you're using the correct chip with `espflash board-info`

### ESP Toolchain Issues
If you get toolchain errors, reinstall:
```bash
espup uninstall
espup install
. $HOME/export-esp.sh
```

## Performance

The ESP32-S3 is significantly more powerful than C3/C6:
- Dual-core Xtensa LX7 @ 240 MHz
- 512 KB SRAM (vs 400 KB on C3)
- Better for complex simulations and more boids

You can increase `NUM_BOIDS` to 30-50 for smoother simulations on S3.

## License

MIT License (same as parent project)
