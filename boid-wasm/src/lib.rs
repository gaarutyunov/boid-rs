use boid_core::{Boid, FlockStd, Vector2D};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlVideoElement};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
pub struct BoidSimulation {
    flock: FlockStd,
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
    pointer_position: Option<Vector2D>,
    pointer_pressed: bool,
    thumb_position: Option<Vector2D>,
    index_position: Option<Vector2D>,
    video_element: Option<HtmlVideoElement>,
    wander_enabled: bool,
    baseline_separation_weight: f32,
    baseline_max_speed: f32,
}

// Pinch detection threshold in pixels
const PINCH_THRESHOLD: f32 = 50.0;
// Maximum distance for scaling parameters (in pixels)
const MAX_FINGER_DISTANCE: f32 = 300.0;

#[wasm_bindgen]
impl BoidSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        canvas_id: &str,
        width: f64,
        height: f64,
        boid_count: usize,
    ) -> Result<BoidSimulation, JsValue> {
        console_log!("Initializing boid simulation with {} boids", boid_count);

        let window = web_sys::window().ok_or("no global window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or("canvas not found")?
            .dyn_into::<HtmlCanvasElement>()?;

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        let context = canvas
            .get_context("2d")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()?;

        let flock = FlockStd::new(width as f32, height as f32, boid_count);

        // Store baseline values for dynamic adjustment
        let baseline_separation_weight = flock.config.separation_weight;
        let baseline_max_speed = flock.config.max_speed;

        Ok(BoidSimulation {
            flock,
            canvas,
            context,
            pointer_position: None,
            pointer_pressed: false,
            thumb_position: None,
            index_position: None,
            video_element: None,
            wander_enabled: false,
            baseline_separation_weight,
            baseline_max_speed,
        })
    }

    pub fn update(&mut self) {
        let target;

        // Check if hand tracking is active
        if let (Some(thumb), Some(index)) = (self.thumb_position, self.index_position) {
            // Calculate distance between thumb and index finger
            let dx = index.x - thumb.x;
            let dy = index.y - thumb.y;
            let distance = (dx * dx + dy * dy).sqrt();

            if distance < PINCH_THRESHOLD {
                // Fingers are pinched - follow the midpoint between fingers
                let midpoint = Vector2D::new(
                    (thumb.x + index.x) / 2.0,
                    (thumb.y + index.y) / 2.0,
                );
                target = Some(midpoint);
                console_log!("Pinch detected! Distance: {:.1}px", distance);
            } else {
                // Fingers are open - adjust separation and speed based on distance
                target = None;

                // Normalize distance (0.0 to 1.0) based on MAX_FINGER_DISTANCE
                let normalized_distance = (distance / MAX_FINGER_DISTANCE).min(1.0);

                // Scale separation weight: larger distance = more separation
                // Range: baseline to baseline * 3
                self.flock.config.separation_weight =
                    self.baseline_separation_weight * (1.0 + normalized_distance * 2.0);

                // Scale max speed: larger distance = faster movement
                // Range: baseline to baseline * 2.5
                self.flock.config.max_speed =
                    self.baseline_max_speed * (1.0 + normalized_distance * 1.5);

                console_log!(
                    "Open fingers - Distance: {:.1}px, Separation: {:.2}, Speed: {:.2}",
                    distance,
                    self.flock.config.separation_weight,
                    self.flock.config.max_speed
                );
            }
        } else {
            // No hand detected - restore baseline values and check for mouse/touch pointer
            self.flock.config.separation_weight = self.baseline_separation_weight;
            self.flock.config.max_speed = self.baseline_max_speed;

            target = if self.pointer_pressed {
                self.pointer_position
            } else {
                None
            };
        }

        self.flock.update_with_target(target);
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Draw video as background if available
        if let Some(ref video) = self.video_element {
            // Draw video flipped horizontally (mirror effect)
            self.context.save();
            self.context.translate(width, 0.0)?;
            self.context.scale(-1.0, 1.0)?;
            self.context
                .draw_image_with_html_video_element_and_dw_and_dh(video, 0.0, 0.0, width, height)?;
            self.context.restore();

            // Add semi-transparent overlay for better boid visibility
            self.context.set_fill_style_str("rgba(10, 10, 10, 0.3)");
            self.context.fill_rect(0.0, 0.0, width, height);
        } else {
            // Clear canvas with dark background if no video
            self.context.set_fill_style_str("#0a0a0a");
            self.context.fill_rect(0.0, 0.0, width, height);
        }

        // Draw finger landmarks if available
        if let (Some(thumb), Some(index)) = (self.thumb_position, self.index_position) {
            self.draw_finger_landmarks(thumb, index)?;
        }

        // Draw each boid
        for boid in &self.flock.boids {
            self.draw_boid(boid)?;
        }

        Ok(())
    }

    fn draw_boid(&self, boid: &Boid) -> Result<(), JsValue> {
        let size = 8.0;
        let angle = (boid.velocity.y as f64).atan2(boid.velocity.x as f64);

        self.context.save();
        self.context
            .translate(boid.position.x as f64, boid.position.y as f64)?;
        self.context.rotate(angle)?;

        // Draw a triangle pointing in the direction of movement
        self.context.begin_path();
        self.context.move_to(size, 0.0);
        self.context.line_to(-size / 2.0, size / 2.0);
        self.context.line_to(-size / 2.0, -size / 2.0);
        self.context.close_path();

        // Fill with gradient color based on velocity
        let speed = boid.velocity.magnitude();
        let normalized_speed = ((speed / self.flock.config.max_speed).min(1.0)) as f64;
        let hue = 180.0 + normalized_speed * 60.0; // Cyan to green
        let color = format!("hsl({}, 70%, 60%)", hue);

        self.context.set_fill_style_str(&color);
        self.context.fill();

        // Outline
        self.context
            .set_stroke_style_str("rgba(255, 255, 255, 0.3)");
        self.context.set_line_width(1.0);
        self.context.stroke();

        self.context.restore();

        Ok(())
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.flock.resize(width as f32, height as f32);
        console_log!("Resized to {}x{}", width, height);
    }

    pub fn boid_count(&self) -> usize {
        self.flock.boids.len()
    }

    pub fn set_separation_weight(&mut self, weight: f64) {
        self.flock.config.separation_weight = weight as f32;
        self.baseline_separation_weight = weight as f32;
    }

    pub fn set_alignment_weight(&mut self, weight: f64) {
        self.flock.config.alignment_weight = weight as f32;
    }

    pub fn set_cohesion_weight(&mut self, weight: f64) {
        self.flock.config.cohesion_weight = weight as f32;
    }

    pub fn set_max_speed(&mut self, speed: f64) {
        self.flock.config.max_speed = speed as f32;
        self.baseline_max_speed = speed as f32;
    }

    pub fn set_max_force(&mut self, force: f64) {
        self.flock.config.max_force = force as f32;
    }

    pub fn set_seek_weight(&mut self, weight: f64) {
        self.flock.config.seek_weight = weight as f32;
    }

    pub fn set_wander_radius(&mut self, radius: f64) {
        self.flock.config.wander_radius = radius as f32;
    }

    pub fn set_wander_enabled(&mut self, enabled: bool) {
        self.wander_enabled = enabled;
        self.flock.config.wander_enabled = enabled;
        console_log!(
            "Wander behavior {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    pub fn get_wander_enabled(&self) -> bool {
        self.wander_enabled
    }

    pub fn handle_pointer_down(&mut self, x: f64, y: f64) {
        self.pointer_position = Some(Vector2D::new(x as f32, y as f32));
        self.pointer_pressed = true;
        console_log!("Pointer down at ({}, {})", x, y);
    }

    pub fn handle_pointer_move(&mut self, x: f64, y: f64) {
        self.pointer_position = Some(Vector2D::new(x as f32, y as f32));
    }

    pub fn handle_pointer_up(&mut self) {
        self.pointer_pressed = false;
        console_log!("Pointer released");
    }

    pub fn get_average_position(&self) -> Option<Vec<f64>> {
        if self.flock.boids.is_empty() {
            return None;
        }

        let mut sum_x = 0.0_f32;
        let mut sum_y = 0.0_f32;
        for boid in &self.flock.boids {
            sum_x += boid.position.x;
            sum_y += boid.position.y;
        }

        let len = self.flock.boids.len() as f32;
        Some(vec![(sum_x / len) as f64, (sum_y / len) as f64])
    }

    pub fn all_boids_within_bounds(&self, width: f64, height: f64) -> bool {
        self.flock.boids.iter().all(|boid| {
            boid.position.x >= 0.0
                && boid.position.x <= width as f32
                && boid.position.y >= 0.0
                && boid.position.y <= height as f32
        })
    }

    pub fn set_video_element(&mut self, video_id: &str) -> Result<(), JsValue> {
        let window = web_sys::window().ok_or("no global window")?;
        let document = window.document().ok_or("no document")?;
        let video = document
            .get_element_by_id(video_id)
            .ok_or("video element not found")?
            .dyn_into::<HtmlVideoElement>()?;
        self.video_element = Some(video);
        console_log!("Video element set");
        Ok(())
    }

    pub fn update_finger_positions(
        &mut self,
        thumb_x: f64,
        thumb_y: f64,
        index_x: f64,
        index_y: f64,
    ) {
        let canvas_width = self.canvas.width() as f32;
        // Mirror the x-coordinates to match the flipped video
        self.thumb_position = Some(Vector2D::new(canvas_width - thumb_x as f32, thumb_y as f32));
        self.index_position = Some(Vector2D::new(canvas_width - index_x as f32, index_y as f32));
    }

    pub fn clear_finger_positions(&mut self) {
        self.thumb_position = None;
        self.index_position = None;
    }

    pub fn get_finger_distance(&self) -> Option<f64> {
        if let (Some(thumb), Some(index)) = (self.thumb_position, self.index_position) {
            let dx = index.x - thumb.x;
            let dy = index.y - thumb.y;
            Some(((dx * dx + dy * dy) as f64).sqrt())
        } else {
            None
        }
    }

    pub fn is_pinched(&self) -> bool {
        if let Some(distance) = self.get_finger_distance() {
            distance < PINCH_THRESHOLD as f64
        } else {
            false
        }
    }

    pub fn get_current_separation_weight(&self) -> f64 {
        self.flock.config.separation_weight as f64
    }

    pub fn get_current_max_speed(&self) -> f64 {
        self.flock.config.max_speed as f64
    }

    fn draw_finger_landmarks(&self, thumb: Vector2D, index: Vector2D) -> Result<(), JsValue> {
        // Draw line between thumb and index
        self.context.begin_path();
        self.context.move_to(thumb.x as f64, thumb.y as f64);
        self.context.line_to(index.x as f64, index.y as f64);
        self.context.set_stroke_style_str("rgba(0, 255, 0, 0.8)");
        self.context.set_line_width(3.0);
        self.context.stroke();

        // Draw thumb circle
        self.context.begin_path();
        self.context.arc(
            thumb.x as f64,
            thumb.y as f64,
            8.0,
            0.0,
            2.0 * std::f64::consts::PI,
        )?;
        self.context.set_fill_style_str("rgba(255, 0, 0, 0.8)");
        self.context.fill();

        // Draw index finger circle
        self.context.begin_path();
        self.context.arc(
            index.x as f64,
            index.y as f64,
            8.0,
            0.0,
            2.0 * std::f64::consts::PI,
        )?;
        self.context.set_fill_style_str("rgba(0, 0, 255, 0.8)");
        self.context.fill();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    fn create_test_canvas() -> Result<HtmlCanvasElement, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .create_element("canvas")?
            .dyn_into::<HtmlCanvasElement>()?;
        canvas.set_id("test-canvas");
        canvas.set_width(800);
        canvas.set_height(600);
        document.body().unwrap().append_child(&canvas)?;
        Ok(canvas)
    }

    fn create_test_simulation() -> Result<BoidSimulation, JsValue> {
        create_test_canvas()?;
        BoidSimulation::new("test-canvas", 800.0, 600.0, 10)
    }

    #[wasm_bindgen_test]
    fn test_simulation_creation() {
        let simulation = create_test_simulation();
        assert!(simulation.is_ok());
        let sim = simulation.unwrap();
        assert_eq!(sim.boid_count(), 10);
    }

    #[wasm_bindgen_test]
    fn test_pointer_down_sets_state() {
        let mut sim = create_test_simulation().unwrap();

        // Initially, pointer should not be pressed
        assert!(!sim.pointer_pressed);
        assert!(sim.pointer_position.is_none());

        // After pointer down, state should be updated
        sim.handle_pointer_down(100.0, 200.0);
        assert!(sim.pointer_pressed);
        assert!(sim.pointer_position.is_some());

        let pos = sim.pointer_position.unwrap();
        assert_eq!(pos.x, 100.0);
        assert_eq!(pos.y, 200.0);
    }

    #[wasm_bindgen_test]
    fn test_pointer_move_updates_position() {
        let mut sim = create_test_simulation().unwrap();

        sim.handle_pointer_down(100.0, 100.0);
        sim.handle_pointer_move(200.0, 300.0);

        assert!(sim.pointer_pressed);
        let pos = sim.pointer_position.unwrap();
        assert_eq!(pos.x, 200.0);
        assert_eq!(pos.y, 300.0);
    }

    #[wasm_bindgen_test]
    fn test_pointer_up_clears_pressed_state() {
        let mut sim = create_test_simulation().unwrap();

        sim.handle_pointer_down(100.0, 100.0);
        assert!(sim.pointer_pressed);

        sim.handle_pointer_up();
        assert!(!sim.pointer_pressed);
        // Position should still be set, just not pressed
        assert!(sim.pointer_position.is_some());
    }

    #[wasm_bindgen_test]
    fn test_update_without_target() {
        let mut sim = create_test_simulation().unwrap();

        // Get initial positions
        let initial_count = sim.boid_count();

        // Update without pointer pressed
        sim.update();

        // Boids should still exist and move
        assert_eq!(sim.boid_count(), initial_count);
    }

    #[wasm_bindgen_test]
    fn test_update_with_target() {
        let mut sim = create_test_simulation().unwrap();

        // Set a target by pressing pointer
        sim.handle_pointer_down(400.0, 300.0);

        // Update should apply seek behavior towards target
        sim.update();

        // Boid count should remain the same
        assert_eq!(sim.boid_count(), 10);
    }

    #[wasm_bindgen_test]
    fn test_configuration_setters() {
        let mut sim = create_test_simulation().unwrap();

        sim.set_separation_weight(2.5);
        assert_eq!(sim.flock.config.separation_weight, 2.5);

        sim.set_alignment_weight(1.8);
        assert_eq!(sim.flock.config.alignment_weight, 1.8);

        sim.set_cohesion_weight(1.2);
        assert_eq!(sim.flock.config.cohesion_weight, 1.2);

        sim.set_max_speed(6.0);
        assert_eq!(sim.flock.config.max_speed, 6.0);

        sim.set_max_force(0.2);
        assert_eq!(sim.flock.config.max_force, 0.2);

        sim.set_seek_weight(10.0);
        assert_eq!(sim.flock.config.seek_weight, 10.0);
    }

    #[wasm_bindgen_test]
    fn test_resize() {
        let mut sim = create_test_simulation().unwrap();

        sim.resize(1024.0, 768.0);

        assert_eq!(sim.canvas.width(), 1024);
        assert_eq!(sim.canvas.height(), 768);
        assert_eq!(sim.flock.width, 1024.0);
        assert_eq!(sim.flock.height, 768.0);
    }

    #[wasm_bindgen_test]
    fn test_get_average_position() {
        let sim = create_test_simulation().unwrap();

        let avg_pos = sim.get_average_position();
        assert!(avg_pos.is_some());

        let pos = avg_pos.unwrap();
        assert_eq!(pos.len(), 2);

        // Average position should be within canvas bounds
        assert!(pos[0] >= 0.0 && pos[0] <= 800.0);
        assert!(pos[1] >= 0.0 && pos[1] <= 600.0);
    }

    #[wasm_bindgen_test]
    fn test_all_boids_within_bounds() {
        let mut sim = create_test_simulation().unwrap();

        // Run several updates to let boids move
        for _ in 0..100 {
            sim.update();
        }

        // All boids should still be within bounds
        assert!(sim.all_boids_within_bounds(800.0, 600.0));
    }

    #[wasm_bindgen_test]
    fn test_render() {
        let sim = create_test_simulation().unwrap();

        // Render should not panic
        let result = sim.render();
        assert!(result.is_ok());
    }
}
