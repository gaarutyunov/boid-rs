use anyhow::{Context, Result};
use boid_shared::{Position, TargetPositionUpdate};
use clap::Parser;
use opencv::{
    core::{Mat, Point, Scalar},
    highgui, imgproc,
    prelude::*,
    videoio::{self, VideoCapture, VideoCaptureAPIs},
};
use std::time::Instant;

mod hand_tracker;
use hand_tracker::HandTracker;

#[derive(Parser, Debug)]
#[command(author, version, about = "Boid client with hand tracking", long_about = None)]
struct Args {
    /// ESP32 server URL (e.g., http://192.168.1.100)
    #[arg(short, long)]
    server: String,

    /// Video stream source: 'esp32' to stream from ESP32 camera, or camera device ID (e.g., '0' for local camera)
    #[arg(short = 'v', long, default_value = "esp32")]
    video_source: String,

    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Show camera window
    #[arg(short = 'w', long, default_value = "true")]
    show_window: bool,
}

struct BoidClient {
    server_url: String,
    camera: VideoCapture,
    hand_tracker: HandTracker,
    http_client: reqwest::blocking::Client,
    last_position: Option<Position>,
    show_window: bool,
}

impl BoidClient {
    fn new(server_url: String, video_source: &str, show_window: bool) -> Result<Self> {
        let camera = if video_source == "esp32" {
            // Stream from ESP32 camera via MJPEG endpoint
            let stream_url = format!("{}/stream", server_url);
            log::info!("Opening ESP32 camera stream from {}...", stream_url);

            let cam = VideoCapture::from_file(&stream_url, VideoCaptureAPIs::CAP_ANY as i32)?;

            if !cam.is_opened()? {
                anyhow::bail!(
                    "Failed to open ESP32 camera stream at {}. \
                    Make sure the ESP32 is running and camera streaming is enabled.",
                    stream_url
                );
            }

            log::info!("Successfully connected to ESP32 camera stream");
            cam
        } else {
            // Use local camera device
            let camera_id: i32 = video_source.parse()
                .context("Video source must be 'esp32' or a camera device ID (e.g., '0')")?;

            log::info!("Opening local camera device {}...", camera_id);
            let mut cam = VideoCapture::new(camera_id, VideoCaptureAPIs::CAP_ANY as i32)?;

            if !cam.is_opened()? {
                anyhow::bail!("Failed to open camera device {}", camera_id);
            }

            // Set camera properties for better performance
            cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0)?;
            cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0)?;

            log::info!("Successfully opened local camera");
            cam
        };

        log::info!("Initializing hand tracker...");
        let hand_tracker = HandTracker::new()?;

        let http_client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()?;

        Ok(Self {
            server_url,
            camera,
            hand_tracker,
            http_client,
            last_position: None,
            show_window,
        })
    }

    fn send_position_update(&mut self, position: Option<Position>) -> Result<()> {
        // Only send if position changed significantly (reduce network traffic)
        if let Some(pos) = position {
            if let Some(last) = self.last_position {
                let distance = ((pos.x - last.x).powi(2) + (pos.y - last.y).powi(2)).sqrt();
                if distance < 5.0 {
                    // Skip update if movement is too small
                    return Ok(());
                }
            }
        }

        let update = TargetPositionUpdate { position };
        let url = format!("{}/api/position", self.server_url);

        match self.http_client.post(&url).json(&update).send() {
            Ok(response) => {
                if response.status().is_success() {
                    self.last_position = position;
                    log::debug!("Position update sent: {:?}", position);
                } else {
                    log::warn!("Server returned error: {}", response.status());
                }
            }
            Err(e) => {
                log::warn!("Failed to send position update: {}", e);
            }
        }

        Ok(())
    }

    fn run(&mut self) -> Result<()> {
        log::info!("Starting main loop...");

        if self.show_window {
            highgui::named_window("Boid Hand Tracker", highgui::WINDOW_AUTOSIZE)?;
        }

        let mut frame = Mat::default();
        let mut frame_count = 0;
        let mut last_fps_time = Instant::now();
        let mut fps = 0.0;

        loop {
            // Capture frame
            self.camera.read(&mut frame)?;
            if frame.empty() {
                log::warn!("Empty frame received");
                continue;
            }

            // Process hand tracking
            let hand_result = self.hand_tracker.process_frame(&frame)?;

            // Send position update to ESP32
            if let Some(ref hand_data) = hand_result {
                let position = Position::new(hand_data.index_tip.x, hand_data.index_tip.y);
                self.send_position_update(Some(position))?;
            } else {
                // No hand detected, clear target
                if self.last_position.is_some() {
                    self.send_position_update(None)?;
                }
            }

            // Draw visualization
            if self.show_window {
                let mut display_frame = frame.clone();

                if let Some(ref hand_data) = hand_result {
                    // Draw finger landmarks
                    imgproc::circle(
                        &mut display_frame,
                        Point::new(hand_data.thumb_tip.x as i32, hand_data.thumb_tip.y as i32),
                        8,
                        Scalar::new(0.0, 0.0, 255.0, 0.0), // Red for thumb
                        -1,
                        imgproc::LINE_8,
                        0,
                    )?;

                    imgproc::circle(
                        &mut display_frame,
                        Point::new(hand_data.index_tip.x as i32, hand_data.index_tip.y as i32),
                        8,
                        Scalar::new(255.0, 0.0, 0.0, 0.0), // Blue for index
                        -1,
                        imgproc::LINE_8,
                        0,
                    )?;

                    // Draw line between fingers
                    imgproc::line(
                        &mut display_frame,
                        Point::new(hand_data.thumb_tip.x as i32, hand_data.thumb_tip.y as i32),
                        Point::new(hand_data.index_tip.x as i32, hand_data.index_tip.y as i32),
                        Scalar::new(0.0, 255.0, 0.0, 0.0), // Green line
                        3,
                        imgproc::LINE_8,
                        0,
                    )?;

                    // Display pinch distance
                    let distance = hand_data.pinch_distance();
                    let text = format!("Distance: {:.1}px", distance);
                    imgproc::put_text(
                        &mut display_frame,
                        &text,
                        Point::new(10, 30),
                        imgproc::FONT_HERSHEY_SIMPLEX,
                        1.0,
                        Scalar::new(0.0, 255.0, 0.0, 0.0),
                        2,
                        imgproc::LINE_8,
                        false,
                    )?;
                }

                // Display FPS
                frame_count += 1;
                if last_fps_time.elapsed().as_secs() >= 1 {
                    fps = frame_count as f64 / last_fps_time.elapsed().as_secs_f64();
                    frame_count = 0;
                    last_fps_time = Instant::now();
                }

                let fps_text = format!("FPS: {:.1}", fps);
                imgproc::put_text(
                    &mut display_frame,
                    &fps_text,
                    Point::new(10, 60),
                    imgproc::FONT_HERSHEY_SIMPLEX,
                    1.0,
                    Scalar::new(255.0, 255.0, 255.0, 0.0),
                    2,
                    imgproc::LINE_8,
                    false,
                )?;

                highgui::imshow("Boid Hand Tracker", &display_frame)?;
            }

            // Check for 'q' key to quit
            if highgui::wait_key(1)? == b'q' as i32 {
                log::info!("Quit requested");
                break;
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Debug)
            .init();
    } else {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    log::info!("Boid client starting...");
    log::info!("Server: {}", args.server);
    log::info!("Video source: {}", args.video_source);

    let mut client = BoidClient::new(args.server, &args.video_source, args.show_window)
        .context("Failed to initialize client")?;

    client.run().context("Client error")?;

    Ok(())
}
