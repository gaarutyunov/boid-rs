// Camera module for XIAO ESP32S3 Sense
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

// Camera driver implementation adapted from:
// https://github.com/Kezii/esp32cam_rs
// Copyright (c) Kezii
// Used under MIT license with attribution as required

use std::marker::PhantomData;

use esp_idf_hal::gpio::*;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_sys::{self as sys, camera, esp, EspError};

pub struct FrameBuffer<'a> {
    fb: *mut camera::camera_fb_t,
    _p: PhantomData<&'a camera::camera_fb_t>,
}

impl<'a> FrameBuffer<'a> {
    pub fn data(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts((*self.fb).buf, (*self.fb).len) }
    }

    pub fn width(&self) -> usize {
        unsafe { (*self.fb).width }
    }

    pub fn height(&self) -> usize {
        unsafe { (*self.fb).height }
    }

    pub fn format(&self) -> camera::pixformat_t {
        unsafe { (*self.fb).format }
    }

    pub fn timestamp(&self) -> camera::timeval {
        unsafe { (*self.fb).timestamp }
    }

    pub fn fb_return(&self) {
        unsafe { camera::esp_camera_fb_return(self.fb) }
    }
}

impl Drop for FrameBuffer<'_> {
    fn drop(&mut self) {
        self.fb_return();
    }
}

pub struct Camera<'a> {
    _p: PhantomData<&'a ()>,
}

impl<'a> Camera<'a> {
    pub fn new(
        pin_pwdn: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_xclk: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d0: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d1: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d2: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d3: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_d4: impl Peripheral<P = impl InputPin> + 'a,
        pin_d5: impl Peripheral<P = impl InputPin> + 'a,
        pin_d6: impl Peripheral<P = impl InputPin> + 'a,
        pin_d7: impl Peripheral<P = impl InputPin> + 'a,
        pin_vsync: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_href: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_pclk: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_sda: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pin_scl: impl Peripheral<P = impl InputPin + OutputPin> + 'a,
        pixel_format: camera::pixformat_t,
        frame_size: camera::framesize_t,
    ) -> Result<Self, EspError> {
        esp_idf_hal::into_ref!(
            pin_pwdn, pin_xclk, pin_d0, pin_d1, pin_d2, pin_d3, pin_d4, pin_d5, pin_d6, pin_d7,
            pin_vsync, pin_href, pin_pclk, pin_sda, pin_scl
        );

        let config = camera::camera_config_t {
            pin_pwdn: pin_pwdn.pin(),
            pin_xclk: pin_xclk.pin(),
            pin_reset: -1,

            pin_d0: pin_d0.pin(),
            pin_d1: pin_d1.pin(),
            pin_d2: pin_d2.pin(),
            pin_d3: pin_d3.pin(),
            pin_d4: pin_d4.pin(),
            pin_d5: pin_d5.pin(),
            pin_d6: pin_d6.pin(),
            pin_d7: pin_d7.pin(),
            pin_vsync: pin_vsync.pin(),
            pin_href: pin_href.pin(),
            pin_pclk: pin_pclk.pin(),

            xclk_freq_hz: 20000000,
            ledc_timer: sys::ledc_timer_t_LEDC_TIMER_0,
            ledc_channel: sys::ledc_channel_t_LEDC_CHANNEL_0,

            pixel_format,
            frame_size,

            jpeg_quality: 12,
            fb_count: 1,
            grab_mode: camera::camera_grab_mode_t_CAMERA_GRAB_WHEN_EMPTY,

            fb_location: camera::camera_fb_location_t_CAMERA_FB_IN_PSRAM,

            __bindgen_anon_1: camera::camera_config_t__bindgen_ty_1 {
                pin_sccb_sda: pin_sda.pin(),
            },
            __bindgen_anon_2: camera::camera_config_t__bindgen_ty_2 {
                pin_sccb_scl: pin_scl.pin(),
            },

            ..Default::default()
        };

        esp!(unsafe { camera::esp_camera_init(&config) })?;
        Ok(Self { _p: PhantomData })
    }

    pub fn get_framebuffer(&self) -> Option<FrameBuffer> {
        let fb = unsafe { camera::esp_camera_fb_get() };
        if fb.is_null() {
            None
        } else {
            Some(FrameBuffer {
                fb,
                _p: PhantomData,
            })
        }
    }
}

impl<'a> Drop for Camera<'a> {
    fn drop(&mut self) {
        esp!(unsafe { camera::esp_camera_deinit() }).expect("error during esp_camera_deinit")
    }
}

// Wrapper for XIAO ESP32S3 Sense specific pin configuration
pub struct CameraWrapper {
    camera: Camera<'static>,
}

#[derive(Debug)]
pub enum CameraError {
    EspError(EspError),
    NoFrameBuffer,
}

impl From<EspError> for CameraError {
    fn from(err: EspError) -> Self {
        CameraError::EspError(err)
    }
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
    ) -> Result<Self, CameraError> {
        log::info!("Initializing camera for XIAO ESP32S3 Sense");

        // Create a dummy PWDN pin - XIAO doesn't use it
        // We'll use GPIO0 which won't actually be used by the camera
        use esp_idf_hal::peripherals::Peripherals;
        let peripherals = Peripherals::take().unwrap();

        let camera = Camera::new(
            peripherals.pins.gpio0, // PWDN (not used on XIAO)
            xclk,                    // GPIO10 - XCLK
            y2,                      // GPIO15 - Y2/D0
            y3,                      // GPIO17 - Y3/D1
            y4,                      // GPIO18 - Y4/D2
            y5,                      // GPIO16 - Y5/D3
            y6,                      // GPIO14 - Y6/D4
            y7,                      // GPIO12 - Y7/D5
            y8,                      // GPIO11 - Y8/D6
            y9,                      // GPIO48 - Y9/D7
            vsync,                   // GPIO38 - VSYNC
            href,                    // GPIO47 - HREF
            pclk,                    // GPIO13 - PCLK
            siod,                    // GPIO40 - SDA
            sioc,                    // GPIO39 - SCL
            camera::pixformat_t_PIXFORMAT_JPEG,
            camera::framesize_t_FRAMESIZE_QVGA, // 320x240
        )?;

        log::info!("Camera initialized successfully");
        Ok(Self { camera })
    }

    /// Capture a JPEG frame from the camera
    /// Returns the frame buffer as a byte slice
    pub fn capture_jpeg(&mut self) -> Result<&[u8], CameraError> {
        // Capture two frames, discard first for freshness (common practice)
        self.camera.get_framebuffer();

        let fb = self.camera
            .get_framebuffer()
            .ok_or(CameraError::NoFrameBuffer)?;

        Ok(fb.data())
    }
}
