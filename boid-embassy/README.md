# Boid Simulation for ESP32-C3/C6 with Embassy

This project implements a real-time boid flocking simulation on the Xiao Seed ESP32-C3/C6 microcontroller using the Embassy async framework and an ST7789-based LCD display.

## Hardware Requirements

- Xiao Seed ESP32-C3 or ESP32-C6 board
- ST7789 240x240 LCD display (or compatible SPI display)
- Connecting wires

## Pin Configuration

Default pin configuration for Xiao ESP32-C3:
- GPIO8: SPI Clock (SCK)
- GPIO10: SPI Data (MOSI/COPI)
- GPIO9: Chip Select (CS)
- GPIO6: Data/Command (DC)
- GPIO7: Reset (RST)

You can modify these pins in `src/main.rs` if your hardware uses different connections.

## Building

### Prerequisites

1. Install Rust (nightly):
```bash
rustup default nightly
```

2. Add the RISC-V target:
```bash
rustup target add riscv32imc-unknown-none-elf
```

3. Install espflash for flashing:
```bash
cargo install espflash
```

### Build the project

```bash
cd boid-embassy
cargo build --release
```

## Flashing

Connect your ESP32-C3/C6 board via USB and run:

```bash
cargo run --release
```

Or manually flash:
```bash
espflash flash target/riscv32imc-unknown-none-elf/release/boid-embassy --monitor
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

## ESP32-C6 Support

To build for ESP32-C6, update `boid-embassy/Cargo.toml`:

```toml
[dependencies]
esp-hal = { version = "0.21", features = ["esp32c6", "embassy", "embassy-time-timg0", "embassy-executor-thread"] }
esp-backtrace = { version = "0.14", features = ["esp32c6", "panic-handler", "exception-handler", "println"] }
esp-println = { version = "0.12", features = ["esp32c6", "log"] }
```

And update `.cargo/config.toml` if the target triple is different.

## Architecture

- `main.rs`: Main application loop, Embassy executor, display initialization
- `display.rs`: Display driver wrapper for ST7789
- `rng.rs`: Simple pseudo-random number generator for embedded use
- `boid-core`: Core boid algorithm (no_std compatible)

## Troubleshooting

### Display not working
- Check pin connections
- Verify your display is ST7789-based (or modify the display driver)
- Ensure power supply is adequate

### Build errors
- Make sure you're using Rust nightly
- Verify all targets are installed: `rustup target list --installed`
- Check that build-std is working: `cargo build -v`

### Flashing errors
- Install/update espflash: `cargo install espflash --force`
- Check USB connection and permissions
- Try holding the BOOT button while connecting

## License

MIT License (same as parent project)
