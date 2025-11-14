#![no_std]
#![no_main]

use boid_core::{Boid, BoidConfig, Flock, Vector2D};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Triangle},
};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    gpio::Io,
    peripherals::Peripherals,
    prelude::*,
    spi::{master::Spi, SpiMode},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use log::info;

mod display;
mod rng;

use display::DisplayWrapper;
use rng::SimpleRng;

// Display configuration for common LCD screens
const DISPLAY_WIDTH: u32 = 240;
const DISPLAY_HEIGHT: u32 = 240;

// Boid simulation configuration
const NUM_BOIDS: usize = 20;
const BOID_SIZE: u32 = 3;

#[main]
async fn main(_spawner: Spawner) {
    info!("Starting boid simulation on ESP32-S3!");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    esp_hal_embassy::init(&clocks, timg0.timer0);

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // SPI pins for Xiao ESP32-S3 Sense
    // These are common pins for SPI displays
    let sclk = io.pins.gpio8; // SCK
    let mosi = io.pins.gpio9; // MOSI (COPI)
    let cs = io.pins.gpio7; // CS
    let dc = io.pins.gpio4; // DC (Data/Command)
    let rst = io.pins.gpio5; // RST (Reset)

    info!("Initializing SPI...");

    // Initialize SPI
    let spi = Spi::new(peripherals.SPI2, 40.MHz(), SpiMode::Mode0, &clocks)
        .with_sck(sclk)
        .with_mosi(mosi);

    info!("Initializing display...");

    // Initialize display
    let mut display = DisplayWrapper::new(spi, cs.into(), dc.into(), rst.into());

    display.clear(Rgb565::BLACK).ok();

    info!("Display initialized!");

    // Initialize the boid simulation
    let config = BoidConfig {
        max_speed: 2.0,
        max_force: 0.05,
        separation_distance: 15.0,
        alignment_distance: 25.0,
        cohesion_distance: 25.0,
        separation_weight: 1.5,
        alignment_weight: 1.0,
        cohesion_weight: 1.0,
    };

    let mut flock = Flock::<NUM_BOIDS>::new(DISPLAY_WIDTH as f32, DISPLAY_HEIGHT as f32, config);

    // Initialize boids with pseudo-random positions
    let mut rng = SimpleRng::new(12345);
    for _ in 0..NUM_BOIDS {
        let x = rng.next_f32() * DISPLAY_WIDTH as f32;
        let y = rng.next_f32() * DISPLAY_HEIGHT as f32;
        let vx = (rng.next_f32() - 0.5) * 4.0;
        let vy = (rng.next_f32() - 0.5) * 4.0;

        let boid = Boid::new(Vector2D::new(x, y), Vector2D::new(vx, vy));
        let _ = flock.add_boid(boid);
    }

    info!("Boids initialized, starting simulation loop...");

    // Main simulation loop
    loop {
        // Clear the display
        display.clear(Rgb565::BLACK).ok();

        // Update boid positions
        flock.update();

        // Draw each boid
        for boid in flock.boids.iter() {
            draw_boid(&mut display, boid);
        }

        // Wait before next frame (targeting ~30 FPS)
        Timer::after(Duration::from_millis(33)).await;
    }
}

fn draw_boid(display: &mut DisplayWrapper, boid: &Boid) {
    let x = boid.position.x as i32;
    let y = boid.position.y as i32;

    // Calculate boid direction for triangle orientation
    let vel_mag = boid.velocity.magnitude();
    if vel_mag > 0.1 {
        let angle = libm::atan2f(boid.velocity.y, boid.velocity.x);

        // Draw a triangle pointing in the direction of movement
        let size = BOID_SIZE as f32;
        let cos_a = libm::cosf(angle);
        let sin_a = libm::sinf(angle);

        // Triangle vertices (pointing right initially, then rotated)
        let p1_x = x + (size * 2.0 * cos_a) as i32;
        let p1_y = y + (size * 2.0 * sin_a) as i32;

        let p2_x = x + (size * cos_a - size * sin_a) as i32;
        let p2_y = y + (size * sin_a + size * cos_a) as i32;

        let p3_x = x + (size * cos_a + size * sin_a) as i32;
        let p3_y = y + (size * sin_a - size * cos_a) as i32;

        let triangle = Triangle::new(
            Point::new(p1_x, p1_y),
            Point::new(p2_x, p2_y),
            Point::new(p3_x, p3_y),
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN));

        triangle.draw(display).ok();
    } else {
        // If not moving, just draw a circle
        let circle = Circle::new(
            Point::new(x - BOID_SIZE as i32, y - BOID_SIZE as i32),
            BOID_SIZE * 2,
        )
        .into_styled(PrimitiveStyle::with_fill(Rgb565::GREEN));
        circle.draw(display).ok();
    }
}
