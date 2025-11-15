use boid_shared::{Position, SettingsUpdate, StatusResponse, TargetPositionUpdate};
use embassy_net::tcp::TcpSocket;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use heapless::Vec;
use picoserve::{response::IntoResponse, routing::parse_path_segment};

pub static TARGET_POSITION: Signal<CriticalSectionRawMutex, Option<Position>> =
    Signal::new();

pub static SETTINGS_UPDATE: Signal<CriticalSectionRawMutex, SettingsUpdate> = Signal::new();

/// Simple HTTP response builder
pub struct Response {
    pub status: u16,
    pub body: heapless::Vec<u8, 512>,
}

impl Response {
    pub fn ok(body: &str) -> Self {
        let mut vec = heapless::Vec::new();
        vec.extend_from_slice(body.as_bytes()).ok();
        Self { status: 200, body: vec }
    }

    pub fn json(body: &str) -> Self {
        let mut vec = heapless::Vec::new();
        vec.extend_from_slice(body.as_bytes()).ok();
        Self { status: 200, body: vec }
    }

    pub fn error(status: u16, message: &str) -> Self {
        let mut vec = heapless::Vec::new();
        vec.extend_from_slice(message.as_bytes()).ok();
        Self { status, body: vec }
    }
}

/// Handle POST /api/position endpoint
pub fn handle_position_update(body: &[u8]) -> Response {
    // Parse JSON using serde-json-core
    match serde_json_core::from_slice::<TargetPositionUpdate>(body) {
        Ok((update, _)) => {
            // Signal the main loop to update target position
            TARGET_POSITION.signal(update.position);
            Response::ok(r#"{"status":"ok"}"#)
        }
        Err(_) => Response::error(400, r#"{"error":"Invalid JSON"}"#),
    }
}

/// Handle POST /api/settings endpoint
pub fn handle_settings_update(body: &[u8]) -> Response {
    match serde_json_core::from_slice::<SettingsUpdate>(body) {
        Ok((update, _)) => {
            SETTINGS_UPDATE.signal(update);
            Response::ok(r#"{"status":"ok"}"#)
        }
        Err(_) => Response::error(400, r#"{"error":"Invalid JSON"}"#),
    }
}

/// Handle GET /api/status endpoint
pub fn handle_status(boid_count: usize, fps: u32, target_active: bool) -> Response {
    let status = StatusResponse {
        boid_count,
        fps,
        target_active,
    };

    // Serialize to JSON
    let mut buf = [0u8; 128];
    match serde_json_core::to_slice(&status, &mut buf) {
        Ok(size) => {
            let json_str = core::str::from_utf8(&buf[..size]).unwrap_or("{}");
            Response::json(json_str)
        }
        Err(_) => Response::error(500, r#"{"error":"Serialization failed"}"#),
    }
}

/// Simple HTTP request parser
pub struct HttpRequest<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    pub fn parse(data: &'a [u8]) -> Option<Self> {
        // Find end of request line
        let request_str = core::str::from_utf8(data).ok()?;
        let mut lines = request_str.lines();
        let request_line = lines.next()?;

        // Parse method and path
        let parts: Vec<&str, 3> = request_line.split(' ').take(3).collect();
        if parts.len() < 2 {
            return None;
        }

        let method = parts[0];
        let path = parts[1];

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

/// Format HTTP response
pub fn format_response(response: &Response, buf: &mut [u8]) -> usize {
    let status_text = match response.status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "Unknown",
    };

    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n",
        response.status, status_text, response.body.len()
    );

    let mut written = 0;
    let header_bytes = header.as_bytes();
    let header_len = header_bytes.len().min(buf.len());
    buf[..header_len].copy_from_slice(&header_bytes[..header_len]);
    written += header_len;

    let body_len = response.body.len().min(buf.len() - written);
    buf[written..written + body_len].copy_from_slice(&response.body[..body_len]);
    written += body_len;

    written
}
