use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use boid_core::Vector2D;
use boid_shared::{SettingsUpdate, StatusResponse, TargetPositionUpdate};
use log::{error, info};

use crate::camera::CameraWrapper;
use crate::types::SimulationState;

/// Start the HTTP server on port 80
pub fn start_server(
    camera: Arc<Mutex<CameraWrapper>>,
    sim_state: Arc<Mutex<SimulationState>>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:80")?;
    listener.set_nonblocking(false)?;

    info!("HTTP server listening on port 80");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let camera_clone = camera.clone();
                let sim_state_clone = sim_state.clone();

                // Handle each connection in the same thread (single-threaded server)
                // For ESP32, we don't want to spawn too many threads
                if let Err(e) = handle_client(stream, camera_clone, sim_state_clone) {
                    error!("Error handling client: {:?}", e);
                }
            }
            Err(e) => {
                error!("Connection error: {:?}", e);
            }
        }
    }

    Ok(())
}

fn handle_client(
    mut stream: TcpStream,
    camera: Arc<Mutex<CameraWrapper>>,
    sim_state: Arc<Mutex<SimulationState>>,
) -> anyhow::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let mut buffer = [0u8; 2048];
    let bytes_read = stream.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(());
    }

    // Parse HTTP request
    if let Some(request) = HttpRequest::parse(&buffer[..bytes_read]) {
        info!("Request: {} {}", request.method, request.path);

        match (request.method, request.path) {
            ("GET", "/stream") => {
                handle_mjpeg_stream(stream, camera)?;
            }
            ("POST", "/api/position") => {
                let response = handle_position_update(request.body, &sim_state);
                write_response(&mut stream, &response)?;
            }
            ("POST", "/api/settings") => {
                let response = handle_settings_update(request.body, &sim_state);
                write_response(&mut stream, &response)?;
            }
            ("GET", "/api/status") => {
                let response = handle_status(&sim_state);
                write_response(&mut stream, &response)?;
            }
            _ => {
                let response = Response::error(404, r#"{"error":"Not found"}"#);
                write_response(&mut stream, &response)?;
            }
        }
    }

    Ok(())
}

fn handle_mjpeg_stream(
    mut stream: TcpStream,
    camera: Arc<Mutex<CameraWrapper>>,
) -> anyhow::Result<()> {
    // Send MJPEG header
    let header = b"HTTP/1.1 200 OK\r\n\
                    Content-Type: multipart/x-mixed-replace; boundary=BOUNDARY\r\n\
                    Access-Control-Allow-Origin: *\r\n\
                    Cache-Control: no-cache\r\n\
                    \r\n";
    stream.write_all(header)?;

    // Stream frames continuously
    loop {
        let jpeg_data = {
            let mut cam = camera.lock().unwrap();
            match cam.capture_jpeg() {
                Ok(data) => data.to_vec(),
                Err(e) => {
                    error!("Camera capture error: {:?}", e);
                    break;
                }
            }
        };

        // Write frame boundary and headers
        let frame_header = format!(
            "--BOUNDARY\r\n\
             Content-Type: image/jpeg\r\n\
             Content-Length: {}\r\n\
             \r\n",
            jpeg_data.len()
        );

        if stream.write_all(frame_header.as_bytes()).is_err() {
            break;
        }

        // Write JPEG data
        if stream.write_all(&jpeg_data).is_err() {
            break;
        }

        // Write trailing newline
        if stream.write_all(b"\r\n").is_err() {
            break;
        }

        stream.flush().ok();

        // Small delay between frames (~10 FPS for camera stream)
        std::thread::sleep(Duration::from_millis(100));
    }

    info!("MJPEG stream ended");
    Ok(())
}

fn handle_position_update(
    body: &[u8],
    sim_state: &Arc<Mutex<SimulationState>>,
) -> Response {
    match serde_json::from_slice::<TargetPositionUpdate>(body) {
        Ok(update) => {
            let mut state = sim_state.lock().unwrap();
            state.target_position = update.position.map(|p| Vector2D::new(p.x, p.y));
            Response::ok(r#"{"status":"ok"}"#)
        }
        Err(_) => Response::error(400, r#"{"error":"Invalid JSON"}"#),
    }
}

fn handle_settings_update(
    body: &[u8],
    sim_state: &Arc<Mutex<SimulationState>>,
) -> Response {
    match serde_json::from_slice::<SettingsUpdate>(body) {
        Ok(update) => {
            let mut state = sim_state.lock().unwrap();
            state.config.separation_weight = update.settings.separation_weight;
            state.config.alignment_weight = update.settings.alignment_weight;
            state.config.cohesion_weight = update.settings.cohesion_weight;
            state.config.max_speed = update.settings.max_speed;
            state.config.max_force = update.settings.max_force;
            Response::ok(r#"{"status":"ok"}"#)
        }
        Err(_) => Response::error(400, r#"{"error":"Invalid JSON"}"#),
    }
}

fn handle_status(sim_state: &Arc<Mutex<SimulationState>>) -> Response {
    let state = sim_state.lock().unwrap();
    let status = StatusResponse {
        boid_count: 20, // NUM_BOIDS from main
        fps: 30,
        target_active: state.target_position.is_some(),
    };

    match serde_json::to_string(&status) {
        Ok(json) => Response::json(&json),
        Err(_) => Response::error(500, r#"{"error":"Serialization failed"}"#),
    }
}

fn write_response(stream: &mut TcpStream, response: &Response) -> anyhow::Result<()> {
    let status_text = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let header = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         \r\n",
        response.status, status_text, response.content_type, response.body.len()
    );

    stream.write_all(header.as_bytes())?;
    stream.write_all(&response.body)?;
    stream.flush()?;

    Ok(())
}

struct Response {
    status: u16,
    body: Vec<u8>,
    content_type: &'static str,
}

impl Response {
    fn ok(body: &str) -> Self {
        Self {
            status: 200,
            body: body.as_bytes().to_vec(),
            content_type: "application/json",
        }
    }

    fn json(body: &str) -> Self {
        Self {
            status: 200,
            body: body.as_bytes().to_vec(),
            content_type: "application/json",
        }
    }

    fn error(status: u16, message: &str) -> Self {
        Self {
            status,
            body: message.as_bytes().to_vec(),
            content_type: "application/json",
        }
    }
}

struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    fn parse(data: &'a [u8]) -> Option<Self> {
        let request_str = std::str::from_utf8(data).ok()?;
        let mut lines = request_str.lines();
        let request_line = lines.next()?;

        let mut parts = request_line.split_whitespace();
        let method = parts.next()?;
        let path = parts.next()?;

        // Find body (after \r\n\r\n)
        let body_start = data
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .map(|pos| pos + 4)
            .unwrap_or(data.len());

        let body = &data[body_start..];

        Some(HttpRequest { method, path, body })
    }
}
