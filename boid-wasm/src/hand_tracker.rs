use boid_shared::HandLandmarks;
use wasm_bindgen::prelude::*;
use web_sys::ImageData;

/// Hand tracker using OpenCV.js via JavaScript
pub struct HandTracker {
    min_contour_area: f64,
}

impl HandTracker {
    pub fn new() -> Result<Self, JsValue> {
        Ok(Self {
            min_contour_area: 5000.0,
        })
    }

    /// Process ImageData and detect hand landmarks using OpenCV.js
    /// Returns HandLandmarks if a hand is detected
    pub fn process_frame(&self, image_data: &ImageData) -> Result<Option<HandLandmarks>, JsValue> {
        // Call JavaScript function that uses opencv.js
        let result = process_hand_detection_js(image_data, self.min_contour_area)?;

        if result.is_null() || result.is_undefined() {
            return Ok(None);
        }

        // Parse result from JavaScript
        let obj = js_sys::Object::from(result);
        let thumb_x = js_sys::Reflect::get(&obj, &"thumbX".into())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Invalid thumbX"))? as f32;
        let thumb_y = js_sys::Reflect::get(&obj, &"thumbY".into())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Invalid thumbY"))? as f32;
        let index_x = js_sys::Reflect::get(&obj, &"indexX".into())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Invalid indexX"))? as f32;
        let index_y = js_sys::Reflect::get(&obj, &"indexY".into())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("Invalid indexY"))? as f32;

        Ok(Some(HandLandmarks::new(
            boid_shared::Position::new(thumb_x, thumb_y),
            boid_shared::Position::new(index_x, index_y),
        )))
    }
}

// Import the JavaScript function that does the actual hand detection
#[wasm_bindgen(module = "/www/hand_detection.js")]
extern "C" {
    #[wasm_bindgen(js_name = processHandDetection, catch)]
    fn process_hand_detection_js(image_data: &ImageData, min_area: f64)
        -> Result<JsValue, JsValue>;
}
