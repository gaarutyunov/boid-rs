use boid_core::{Boid, FlockStd, Vector2D};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, Element, HtmlCanvasElement, MouseEvent, TouchEvent};

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
}

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

        Ok(BoidSimulation {
            flock,
            canvas,
            context,
            pointer_position: None,
            pointer_pressed: false,
        })
    }

    pub fn update(&mut self) {
        let target = if self.pointer_pressed {
            self.pointer_position
        } else {
            None
        };
        self.flock.update_with_target(target);
    }

    pub fn render(&self) -> Result<(), JsValue> {
        let width = self.canvas.width() as f64;
        let height = self.canvas.height() as f64;

        // Clear canvas with dark background
        self.context.set_fill_style_str("#0a0a0a");
        self.context.fill_rect(0.0, 0.0, width, height);

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

    pub fn add_boid_at(&mut self, x: f64, y: f64) {
        let position = Vector2D::new(x as f32, y as f32);
        let velocity = Vector2D::new(
            ((js_sys::Math::random() - 0.5) * 4.0) as f32,
            ((js_sys::Math::random() - 0.5) * 4.0) as f32,
        );
        let boid = Boid::new(position, velocity);
        self.flock.add_boid(boid);
        console_log!(
            "Added boid at ({}, {}). Total boids: {}",
            x,
            y,
            self.flock.boids.len()
        );
    }

    pub fn resize(&mut self, width: f64, height: f64) {
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        self.flock.resize(width as f32, height as f32);
        console_log!("Resized to {}x{}", width, height);
    }

    pub fn handle_mouse_click(&mut self, event: MouseEvent) {
        let canvas_element: &Element = self.canvas.as_ref();
        let rect = canvas_element.get_bounding_client_rect();
        let x = event.client_x() as f64 - rect.left();
        let y = event.client_y() as f64 - rect.top();
        self.add_boid_at(x, y);
    }

    pub fn handle_touch(&mut self, event: TouchEvent) {
        let touches = event.touches();
        for i in 0..touches.length() {
            if let Some(touch) = touches.item(i) {
                let canvas_element: &Element = self.canvas.as_ref();
                let rect = canvas_element.get_bounding_client_rect();
                let x = touch.client_x() as f64 - rect.left();
                let y = touch.client_y() as f64 - rect.top();
                self.add_boid_at(x, y);
            }
        }
    }

    pub fn boid_count(&self) -> usize {
        self.flock.boids.len()
    }

    pub fn set_separation_weight(&mut self, weight: f64) {
        self.flock.config.separation_weight = weight as f32;
    }

    pub fn set_alignment_weight(&mut self, weight: f64) {
        self.flock.config.alignment_weight = weight as f32;
    }

    pub fn set_cohesion_weight(&mut self, weight: f64) {
        self.flock.config.cohesion_weight = weight as f32;
    }

    pub fn set_max_speed(&mut self, speed: f64) {
        self.flock.config.max_speed = speed as f32;
    }

    pub fn set_max_force(&mut self, force: f64) {
        self.flock.config.max_force = force as f32;
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
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_basic() {
        // Basic test to ensure the module compiles
        assert_eq!(2 + 2, 4);
    }
}
