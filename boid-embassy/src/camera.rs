// Camera module for XIAO ESP32S3 Sense
//
// This module provides camera support using esp32cam library with ESP-IDF framework.
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

use esp32cam::Camera;
use esp_idf_svc::sys::camera::{
    framesize_t_FRAMESIZE_QVGA,
    pixformat_t_PIXFORMAT_JPEG,
};
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripheral::Peripheral;

pub struct CameraWrapper {
    camera: Camera,
}

impl CameraWrapper {
    pub fn new(
        xclk: impl Peripheral<P = Gpio10> + 'static,
        siod: impl Peripheral<P = Gpio40> + 'static,
        sioc: impl Peripheral<P = Gpio39> + 'static,
        y9: impl Peripheral<P = Gpio48> + 'static,
        y8: impl Peripheral<P = Gpio11> + 'static,
        y7: impl Peripheral<P = Gpio12> + 'static,
        y6: impl Peripheral<P = Gpio14> + 'static,
        y5: impl Peripheral<P = Gpio16> + 'static,
        y4: impl Peripheral<P = Gpio18> + 'static,
        y3: impl Peripheral<P = Gpio17> + 'static,
        y2: impl Peripheral<P = Gpio15> + 'static,
        pclk: impl Peripheral<P = Gpio13> + 'static,
        vsync: impl Peripheral<P = Gpio38> + 'static,
        href: impl Peripheral<P = Gpio47> + 'static,
    ) -> Result<Self, esp32cam::CameraError> {
        log::info!("Initializing camera for XIAO ESP32S3 Sense");

        let camera = Camera::new(
            xclk,   // GPIO10
            siod,   // GPIO40
            sioc,   // GPIO39
            y9,     // GPIO48
            y8,     // GPIO11
            y7,     // GPIO12
            y6,     // GPIO14
            y5,     // GPIO16
            y4,     // GPIO18
            y3,     // GPIO17
            y2,     // GPIO15
            pclk,   // GPIO13
            vsync,  // GPIO38
            href,   // GPIO47
            pixformat_t_PIXFORMAT_JPEG,
            framesize_t_FRAMESIZE_QVGA,  // 320x240
        )?;

        log::info!("Camera initialized successfully");
        Ok(Self { camera })
    }

    /// Capture a JPEG frame from the camera
    /// Returns the frame buffer as a byte slice
    pub fn capture_jpeg(&mut self) -> Result<&[u8], esp32cam::CameraError> {
        // Capture two frames, discard first for freshness (common practice)
        self.camera.get_framebuffer()?;
        let fb = self.camera.get_framebuffer()?;
        Ok(fb)
    }
}
