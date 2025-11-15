#![no_std]
#![no_main]

use boid_core::{Boid, BoidConfig, Flock, Vector2D};
use boid_shared::Position;
use embassy_executor::Spawner;
use embassy_net::{Stack, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, Receiver, Sender};
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
    rng::Rng,
    spi::{master::Spi, SpiMode},
    system::SystemControl,
    timer::timg::TimerGroup,
};
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
    WifiState,
};
use log::info;
use static_cell::StaticCell;

mod display;
mod rng;
mod http_server;
mod wifi_config;

use display::DisplayWrapper;
use rng::SimpleRng;

// Display configuration for common LCD screens
const DISPLAY_WIDTH: u32 = 240;
const DISPLAY_HEIGHT: u32 = 240;

// Boid simulation configuration
const NUM_BOIDS: usize = 20;
const BOID_SIZE: u32 = 3;

// Channels for communication between tasks
static TARGET_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, Option<Position>, 1>> =
    StaticCell::new();
static SETTINGS_CHANNEL: StaticCell<Channel<CriticalSectionRawMutex, boid_shared::SettingsUpdate, 1>> =
    StaticCell::new();

#[main]
async fn main(spawner: Spawner) {
    info!("Starting boid simulation on ESP32-S3!");

    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
    let clocks = ClockControl::max(system.clock_control).freeze();

    let timg0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let timg1 = TimerGroup::new(peripherals.TIMG1, &clocks);

    esp_hal_embassy::init(&clocks, timg0.timer0);

    // Initialize RNG for WiFi
    let rng_peripheral = Rng::new(peripherals.RNG);

    // Initialize WiFi
    let wifi_init = esp_wifi::initialize(
        esp_wifi::EspWifiInitFor::Wifi,
        timg1.timer0,
        rng_peripheral,
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    let wifi = peripherals.WIFI;
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&wifi_init, wifi, WifiStaDevice).unwrap();

    // Initialize network stack
    static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    static STACK: StaticCell<Stack<WifiDevice<'_, WifiStaDevice>>> = StaticCell::new();

    let stack = &*STACK.init(Stack::new(
        wifi_interface,
        embassy_net::Config::dhcpv4(Default::default()),
        STACK_RESOURCES.init(StackResources::new()),
        1234u64, // Random seed
    ));

    // Initialize channels
    let target_channel = TARGET_CHANNEL.init(Channel::new());
    let settings_channel = SETTINGS_CHANNEL.init(Channel::new());

    // Spawn WiFi tasks
    spawner.spawn(wifi_task(controller)).ok();
    spawner.spawn(net_task(stack)).ok();
    spawner.spawn(http_server_task(stack, target_channel.sender(), settings_channel.sender())).ok();

    // Wait for network to be ready
    info!("Waiting for network...");
    while !stack.is_link_up() {
        Timer::after(Duration::from_millis(500)).await;
    }
    info!("Network link is up!");

    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(500)).await;
    }
    info!("Network configured!");

    if let Some(config) = stack.config_v4() {
        info!("IP Address: {}", config.address);
        info!("Connect client to this IP address");
    }

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
    let target_receiver = target_channel.receiver();
    let settings_receiver = settings_channel.receiver();
    let mut target_position: Option<Vector2D> = None;

    info!("Boids initialized, starting simulation loop...");

    loop {
        // Check for target position updates (non-blocking)
        if let Ok(new_target) = target_receiver.try_receive() {
            target_position = new_target.map(|p| Vector2D::new(p.x, p.y));
            info!("Target updated: {:?}", target_position);
        }

        // Check for settings updates (non-blocking)
        if let Ok(settings) = settings_receiver.try_receive() {
            flock.config.separation_weight = settings.settings.separation_weight;
            flock.config.alignment_weight = settings.settings.alignment_weight;
            flock.config.cohesion_weight = settings.settings.cohesion_weight;
            flock.config.max_speed = settings.settings.max_speed;
            flock.config.max_force = settings.settings.max_force;
            info!("Settings updated");
        }

        // Clear the display
        display.clear(Rgb565::BLACK).ok();

        // Update boid positions with optional target
        if let Some(target) = target_position {
            flock.update_with_target(Some(target));
        } else {
            flock.update();
        }

        // Draw each boid
        for boid in flock.boids.iter() {
            draw_boid(&mut display, boid);
        }

        // Wait before next frame (targeting ~30 FPS)
        Timer::after(Duration::from_millis(33)).await;
    }
}

#[embassy_executor::task]
async fn wifi_task(mut controller: WifiController<'static>) {
    use wifi_config::{PASSWORD, SSID};

    info!("Starting WiFi controller...");
    controller.start().await.unwrap();
    info!("WiFi started!");

    let client_config = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        password: PASSWORD.try_into().unwrap(),
        ..Default::default()
    });
    controller.set_configuration(&client_config).unwrap();

    info!("Connecting to WiFi...");
    controller.connect().await.unwrap();
    info!("WiFi connected!");

    loop {
        match controller.wait_for_event().await {
            WifiEvent::StaConnected => info!("WiFi: Station connected"),
            WifiEvent::StaDisconnected => {
                info!("WiFi: Station disconnected, reconnecting...");
                controller.connect().await.ok();
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn http_server_task(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    target_sender: Sender<'static, CriticalSectionRawMutex, Option<Position>, 1>,
    settings_sender: Sender<'static, CriticalSectionRawMutex, boid_shared::SettingsUpdate, 1>,
) {
    use embassy_net::tcp::TcpSocket;
    use heapless::Vec;

    let mut rx_buffer = [0; 2048];
    let mut tx_buffer = [0; 2048];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        info!("HTTP server listening on port 80...");
        if let Err(e) = socket.accept(80).await {
            info!("Accept error: {:?}", e);
            continue;
        }

        info!("Client connected");

        let mut buf = [0u8; 1024];
        loop {
            match socket.read(&mut buf).await {
                Ok(0) => {
                    info!("Client disconnected");
                    break;
                }
                Ok(n) => {
                    // Parse HTTP request
                    if let Some(req) = http_server::HttpRequest::parse(&buf[..n]) {
                        info!("Request: {} {}", req.method, req.path);

                        let response = match (req.method, req.path) {
                            ("POST", "/api/position") => {
                                let resp = http_server::handle_position_update(req.body);
                                // Send to channel if successful
                                if resp.status == 200 {
                                    if let Ok((update, _)) =
                                        serde_json_core::from_slice::<boid_shared::TargetPositionUpdate>(
                                            req.body,
                                        )
                                    {
                                        target_sender.send(update.position).await;
                                    }
                                }
                                resp
                            }
                            ("POST", "/api/settings") => {
                                let resp = http_server::handle_settings_update(req.body);
                                if resp.status == 200 {
                                    if let Ok((update, _)) =
                                        serde_json_core::from_slice::<boid_shared::SettingsUpdate>(
                                            req.body,
                                        )
                                    {
                                        settings_sender.send(update).await;
                                    }
                                }
                                resp
                            }
                            ("GET", "/api/status") => {
                                http_server::handle_status(NUM_BOIDS, 30, true)
                            }
                            _ => http_server::Response::error(404, r#"{"error":"Not found"}"#),
                        };

                        // Format and send response
                        let mut response_buf = [0u8; 512];
                        let size = http_server::format_response(&response, &mut response_buf);
                        if socket.write_all(&response_buf[..size]).await.is_err() {
                            break;
                        }
                    }
                }
                Err(_) => {
                    info!("Read error");
                    break;
                }
            }
        }
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
