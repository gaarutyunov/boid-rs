use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration as StdDuration;

use boid_core::{Boid, BoidConfig, Flock, Vector2D};
use boid_shared::Position;
use embedded_graphics::{
    pixelcolor::Rgb565,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, Triangle},
};
use esp_idf_hal::{
    gpio::PinDriver,
    peripherals::Peripherals,
    spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::*,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

mod camera;
mod display;
mod http_server;
mod rng;
mod types;
mod wifi_config;

use camera::CameraWrapper;
use display::DisplayWrapper;
use rng::SimpleRng;
use types::SimulationState;

// Display configuration for common LCD screens
const DISPLAY_WIDTH: u32 = 240;
const DISPLAY_HEIGHT: u32 = 240;

// Boid simulation configuration
const NUM_BOIDS: usize = 20;
const BOID_SIZE: u32 = 3;

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF services
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Starting boid simulation on ESP32-S3 with camera streaming!");

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // Initialize WiFi
    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
    )?;

    connect_wifi(&mut wifi)?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("WiFi connected!");
    info!("IP Address: {}", ip_info.ip);
    info!("Connect client to: http://{}", ip_info.ip);

    // Initialize camera
    let camera = Arc::new(Mutex::new(CameraWrapper::new(
        peripherals.pins.gpio10, // XCLK
        peripherals.pins.gpio40, // SIOD
        peripherals.pins.gpio39, // SIOC
        peripherals.pins.gpio48, // Y9
        peripherals.pins.gpio11, // Y8
        peripherals.pins.gpio12, // Y7
        peripherals.pins.gpio14, // Y6
        peripherals.pins.gpio16, // Y5
        peripherals.pins.gpio18, // Y4
        peripherals.pins.gpio17, // Y3
        peripherals.pins.gpio15, // Y2
        peripherals.pins.gpio13, // PCLK
        peripherals.pins.gpio38, // VSYNC
        peripherals.pins.gpio47, // HREF
    )?));

    // Initialize SPI for display
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio8,  // SCLK
        peripherals.pins.gpio9,  // MOSI
        Option::<esp_idf_hal::gpio::Gpio0>::None, // MISO (not used)
        Some(peripherals.pins.gpio7), // CS
        &SpiDriverConfig::new(),
        &SpiConfig::new().baudrate(40.MHz().into()),
    )?;

    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio5)?;

    let mut display = DisplayWrapper::new(spi, dc, rst);
    display.clear(Rgb565::BLACK).ok();
    info!("Display initialized!");

    // Initialize shared simulation state
    let sim_state = Arc::new(Mutex::new(SimulationState {
        target_position: None,
        config: BoidConfig {
            max_speed: 2.0,
            max_force: 0.05,
            separation_distance: 15.0,
            alignment_distance: 25.0,
            cohesion_distance: 25.0,
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
        },
    }));

    // Spawn HTTP server thread
    let camera_clone = camera.clone();
    let sim_state_clone = sim_state.clone();
    thread::spawn(move || {
        if let Err(e) = http_server::start_server(camera_clone, sim_state_clone) {
            log::error!("HTTP server error: {:?}", e);
        }
    });

    // Initialize the boid simulation
    let config = {
        let state = sim_state.lock().unwrap();
        state.config.clone()
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
        // Update configuration and target from shared state
        {
            let state = sim_state.lock().unwrap();
            flock.config = state.config.clone();

            // Clear display
            display.clear(Rgb565::BLACK).ok();

            // Update boid positions with optional target
            if let Some(target) = state.target_position {
                flock.update_with_target(Some(target));
            } else {
                flock.update();
            }
        }

        // Draw each boid
        for boid in flock.boids.iter() {
            draw_boid(&mut display, boid);
        }

        // Target ~30 FPS
        thread::sleep(StdDuration::from_millis(33));
    }
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) -> anyhow::Result<()> {
    use wifi_config::{PASSWORD, SSID};

    let wifi_configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration)?;

    info!("Starting WiFi...");
    wifi.start()?;
    info!("WiFi started");

    info!("Connecting to WiFi...");
    wifi.connect()?;
    info!("WiFi connected");

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;
    info!("WiFi netif is up");

    Ok(())
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
