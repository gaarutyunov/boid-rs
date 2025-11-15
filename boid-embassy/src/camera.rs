// Camera module for XIAO ESP32S3 Sense
//
// ⚠️  IMPORTANT COMPATIBILITY NOTE ⚠️
//
// The esp32cam_rs library (recommended for ESP32 camera support) requires:
// - esp-idf-svc (ESP-IDF framework)
// - Rust std library
//
// However, this Embassy-based boid simulation uses no_std for:
// - Smaller binary size
// - Better embedded performance
// - Async/await with Embassy runtime
//
// SOLUTION OPTIONS:
//
// 1. **Hybrid Approach** (Recommended for testing):
//    Create a separate std-based camera server binary that runs alongside
//    the no_std boid simulation. See README_CAMERA.md for details.
//
// 2. **Manual FFI Bindings**:
//    Create no_std-compatible bindings to ESP-IDF camera driver.
//    Complex but allows single binary deployment.
//
// 3. **Use Client Fallback**:
//    Test the full pipeline using client's local camera:
//    `cargo run -- --server http://ESP32_IP --video-source 0`
//
// Pin Configuration for XIAO ESP32S3 Sense (OV2640 camera):
// Based on Seeed Studio documentation
//
// XIAO ESP32S3 Sense Camera Pins:
// - PWDN: -1 (not used, tied to 3V3)
// - RESET: -1 (not used)
// - XCLK: GPIO10
// - SIOD (SDA): GPIO40
// - SIOC (SCL): GPIO39
// - Y9 (D7): GPIO48
// - Y8 (D6): GPIO11
// - Y7 (D5): GPIO12
// - Y6 (D4): GPIO14
// - Y5 (D3): GPIO16
// - Y4 (D2): GPIO18
// - Y3 (D1): GPIO17
// - Y2 (D0): GPIO15
// - VSYNC: GPIO38
// - HREF: GPIO47
// - PCLK: GPIO13

// Reference implementation using esp32cam_rs (requires std):
//
// ```rust
// use esp32cam::Camera;
// use esp_idf_svc::sys::{
//     camera::framesize_t_FRAMESIZE_QVGA,
//     camera::pixformat_t_PIXFORMAT_JPEG,
// };
//
// pub struct CameraWrapper {
//     camera: Camera,
// }
//
// impl CameraWrapper {
//     pub fn new(io: &Io) -> Result<Self, &'static str> {
//         let camera = Camera::new(
//             io.pins.gpio10,  // XCLK
//             io.pins.gpio40,  // SIOD
//             io.pins.gpio39,  // SIOC
//             io.pins.gpio48,  // Y9
//             io.pins.gpio11,  // Y8
//             io.pins.gpio12,  // Y7
//             io.pins.gpio14,  // Y6
//             io.pins.gpio16,  // Y5
//             io.pins.gpio18,  // Y4
//             io.pins.gpio17,  // Y3
//             io.pins.gpio15,  // Y2
//             io.pins.gpio13,  // PCLK
//             io.pins.gpio38,  // VSYNC
//             io.pins.gpio47,  // HREF
//             pixformat_t_PIXFORMAT_JPEG,   // JPEG format
//             framesize_t_FRAMESIZE_QVGA,   // 320x240
//         )?;
//
//         Ok(Self { camera })
//     }
//
//     pub fn capture_jpeg(&mut self) -> Result<&[u8], &'static str> {
//         // Capture two frames, discard first for freshness
//         self.camera.get_framebuffer()?;
//         let fb = self.camera.get_framebuffer()?;
//         Ok(fb)
//     }
// }
// ```
//
// For MJPEG streaming over HTTP, see:
// https://github.com/Kezii/esp32cam_rs/blob/master/examples/webserver.rs

// Placeholder for future no_std camera implementation
pub struct CameraPlaceholder;

impl CameraPlaceholder {
    pub fn new() -> Self {
        log::warn!("Camera support not available in no_std Embassy build");
        log::warn!("See boid-embassy/README_CAMERA.md for implementation options");
        Self
    }
}
