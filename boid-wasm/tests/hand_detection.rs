use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Helper function to load an image from URL and convert to ImageData
async fn load_image_data(url: &str) -> Result<web_sys::ImageData, JsValue> {
    let window = web_sys::window().expect("no global window");
    let document = window.document().expect("no document");

    // Create an image element
    let img = document
        .create_element("img")?
        .dyn_into::<web_sys::HtmlImageElement>()?;

    // Create a promise to wait for image to load
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let img_clone = img.clone();
        let onload = Closure::wrap(Box::new(move || {
            resolve.call0(&JsValue::NULL).unwrap();
        }) as Box<dyn FnMut()>);

        let onerror = Closure::wrap(Box::new(move |_| {
            reject
                .call1(&JsValue::NULL, &JsValue::from_str("Failed to load image"))
                .unwrap();
        }) as Box<dyn FnMut(JsValue)>);

        img_clone.set_onload(Some(onload.as_ref().unchecked_ref()));
        img_clone.set_onerror(Some(onerror.as_ref().unchecked_ref()));

        onload.forget();
        onerror.forget();
    });

    img.set_src(url);

    // Wait for the image to load
    wasm_bindgen_futures::JsFuture::from(promise).await?;

    // Create a canvas to extract ImageData
    let canvas = document
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    canvas.set_width(img.width());
    canvas.set_height(img.height());

    let context = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    context.draw_image_with_html_image_element(&img, 0.0, 0.0)?;

    let image_data = context.get_image_data(0.0, 0.0, img.width() as f64, img.height() as f64)?;

    Ok(image_data)
}

/// Create synthetic test image data with hand-like shape
fn create_synthetic_hand_image(
    width: u32,
    height: u32,
    finger_distance: f32,
) -> web_sys::ImageData {
    let size = (width * height * 4) as usize;
    let mut data = vec![255u8; size]; // White background

    let center_x = (width / 2) as i32;
    let center_y = (height / 2) as i32;

    // Draw palm (skin colored ellipse)
    for y in (center_y + 40)..(center_y + 180) {
        for x in (center_x - 80)..(center_x + 80) {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let dx = x - center_x;
                let dy = y - (center_y + 110);
                if (dx * dx) / (80 * 80) + (dy * dy) / (70 * 70) < 1 {
                    let idx = ((y as u32 * width + x as u32) * 4) as usize;
                    data[idx] = 180; // R
                    data[idx + 1] = 150; // G
                    data[idx + 2] = 120; // B
                    data[idx + 3] = 255; // A
                }
            }
        }
    }

    // Draw thumb (left)
    let thumb_x = center_x - (finger_distance / 2.0) as i32;
    for y in (center_y - 25)..(center_y + 25) {
        for x in (thumb_x - 25)..(thumb_x + 25) {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let dx = x - thumb_x;
                let dy = y - center_y;
                if dx * dx + dy * dy < 625 {
                    // 25^2 = 625
                    let idx = ((y as u32 * width + x as u32) * 4) as usize;
                    data[idx] = 180;
                    data[idx + 1] = 150;
                    data[idx + 2] = 120;
                    data[idx + 3] = 255;
                }
            }
        }
    }

    // Draw index (right)
    let index_x = center_x + (finger_distance / 2.0) as i32;
    for y in (center_y - 25)..(center_y + 25) {
        for x in (index_x - 25)..(index_x + 25) {
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let dx = x - index_x;
                let dy = y - center_y;
                if dx * dx + dy * dy < 625 {
                    let idx = ((y as u32 * width + x as u32) * 4) as usize;
                    data[idx] = 180;
                    data[idx + 1] = 150;
                    data[idx + 2] = 120;
                    data[idx + 3] = 255;
                }
            }
        }
    }

    web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&data),
        width,
        height,
    )
    .unwrap()
}

#[wasm_bindgen_test]
fn test_hand_tracker_initialization() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(&"[TEST] Initializing hand tracker...".into());

    let tracker = HandTracker::new();
    assert!(
        tracker.is_ok(),
        "Hand tracker should initialize successfully"
    );

    web_sys::console::log_1(&"[SUCCESS] Hand tracker initialized ✓".into());
}

#[wasm_bindgen_test]
fn test_synthetic_hand_detection_no_hand() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(&"[TEST] Testing with no hand (blank image)...".into());

    let tracker = HandTracker::new().expect("Failed to create tracker");

    // Create blank image (no skin color)
    let size = 640 * 480 * 4;
    let data = vec![200u8; size]; // Light blue background

    let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(
        wasm_bindgen::Clamped(&data),
        640,
        480,
    )
    .unwrap();

    let result = tracker.process_frame(&image_data);
    assert!(result.is_ok(), "Processing should not error");

    let landmarks = result.unwrap();
    assert!(landmarks.is_none(), "Should not detect hand in blank image");

    web_sys::console::log_1(&"[SUCCESS] No false positives on blank image ✓".into());
}

#[wasm_bindgen_test]
fn test_synthetic_hand_detection_close_pinch() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(&"[TEST] Testing with synthetic close pinch (50px)...".into());

    let tracker = HandTracker::new().expect("Failed to create tracker");
    let image_data = create_synthetic_hand_image(640, 480, 50.0);

    let result = tracker.process_frame(&image_data);
    assert!(result.is_ok(), "Processing should not error");

    // Note: Synthetic images may not always be detected by the skin-color-based algorithm
    // This test is primarily to ensure no crashes occur
    match result.unwrap() {
        Some(landmarks) => {
            web_sys::console::log_1(
                &format!(
                    "[INFO] Hand detected - Distance: {:.2}px",
                    landmarks.pinch_distance()
                )
                .into(),
            );
            assert!(
                landmarks.thumb_tip.x < landmarks.index_tip.x,
                "Thumb should be to the left of index"
            );
        }
        None => {
            web_sys::console::log_1(
                &"[INFO] Hand not detected (expected for synthetic images)".into(),
            );
        }
    }

    web_sys::console::log_1(&"[SUCCESS] Synthetic close pinch test completed ✓".into());
}

#[wasm_bindgen_test]
fn test_synthetic_hand_detection_wide_pinch() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(&"[TEST] Testing with synthetic wide pinch (200px)...".into());

    let tracker = HandTracker::new().expect("Failed to create tracker");
    let image_data = create_synthetic_hand_image(640, 480, 200.0);

    let result = tracker.process_frame(&image_data);
    assert!(result.is_ok(), "Processing should not error");

    match result.unwrap() {
        Some(landmarks) => {
            let distance = landmarks.pinch_distance();
            web_sys::console::log_1(
                &format!("[INFO] Hand detected - Distance: {:.2}px", distance).into(),
            );
            assert!(
                landmarks.thumb_tip.x < landmarks.index_tip.x,
                "Thumb should be to the left of index"
            );
        }
        None => {
            web_sys::console::log_1(
                &"[INFO] Hand not detected (expected for synthetic images)".into(),
            );
        }
    }

    web_sys::console::log_1(&"[SUCCESS] Synthetic wide pinch test completed ✓".into());
}

#[wasm_bindgen_test]
async fn test_real_image_hand_detection_open() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(&"[TEST] Testing with real image: IMG_8522.jpeg (open hand)...".into());

    // Wait for OpenCV.js to be ready
    let cv_ready = js_sys::eval("typeof cv !== 'undefined' && cv.Mat")
        .unwrap()
        .is_truthy();

    if !cv_ready {
        web_sys::console::log_1(&"[SKIP] OpenCV.js not loaded, skipping real image test".into());
        return;
    }

    let tracker = HandTracker::new().expect("Failed to create tracker");

    // Load real image from test_images directory
    let image_url = "/tests/test_images/IMG_8522.jpeg";
    let image_data = match load_image_data(image_url).await {
        Ok(data) => data,
        Err(e) => {
            web_sys::console::log_1(
                &format!("[SKIP] Failed to load image {}: {:?}", image_url, e).into(),
            );
            return;
        }
    };

    web_sys::console::log_1(
        &format!(
            "[INFO] Loaded image: {}x{}",
            image_data.width(),
            image_data.height()
        )
        .into(),
    );

    let result = tracker.process_frame(&image_data);
    assert!(result.is_ok(), "Processing should not error");

    let landmarks = result.unwrap();
    assert!(
        landmarks.is_some(),
        "Hand should be detected in real image (open hand)"
    );

    if let Some(landmarks) = landmarks {
        let distance = landmarks.pinch_distance();
        web_sys::console::log_1(
            &format!(
                "[SUCCESS] Open hand detected - Distance: {:.2}px, Thumb: ({:.2}, {:.2}), Index: ({:.2}, {:.2})",
                distance, landmarks.thumb_tip.x, landmarks.thumb_tip.y, landmarks.index_tip.x, landmarks.index_tip.y
            )
            .into(),
        );

        // Validate positions are reasonable
        assert!(landmarks.thumb_tip.x >= 0.0 && landmarks.thumb_tip.x <= 4032.0);
        assert!(landmarks.thumb_tip.y >= 0.0 && landmarks.thumb_tip.y <= 3024.0);
        assert!(landmarks.index_tip.x >= 0.0 && landmarks.index_tip.x <= 4032.0);
        assert!(landmarks.index_tip.y >= 0.0 && landmarks.index_tip.y <= 3024.0);

        // Open hand should have significant distance
        assert!(
            distance > 100.0,
            "Open hand should have distance > 100px, got {:.2}px",
            distance
        );
    }

    web_sys::console::log_1(&"[SUCCESS] Real image open hand test passed ✓".into());
}

#[wasm_bindgen_test]
async fn test_real_image_hand_detection_closed() {
    use boid_wasm::hand_tracker::HandTracker;

    web_sys::console::log_1(
        &"[TEST] Testing with real image: IMG_8528.jpeg (closed pinch)...".into(),
    );

    // Wait for OpenCV.js to be ready
    let cv_ready = js_sys::eval("typeof cv !== 'undefined' && cv.Mat")
        .unwrap()
        .is_truthy();

    if !cv_ready {
        web_sys::console::log_1(&"[SKIP] OpenCV.js not loaded, skipping real image test".into());
        return;
    }

    let tracker = HandTracker::new().expect("Failed to create tracker");

    let image_url = "/tests/test_images/IMG_8528.jpeg";
    let image_data = match load_image_data(image_url).await {
        Ok(data) => data,
        Err(e) => {
            web_sys::console::log_1(
                &format!("[SKIP] Failed to load image {}: {:?}", image_url, e).into(),
            );
            return;
        }
    };

    let result = tracker.process_frame(&image_data);
    assert!(result.is_ok(), "Processing should not error");

    let landmarks = result.unwrap();
    assert!(
        landmarks.is_some(),
        "Hand should be detected in real image (closed pinch)"
    );

    if let Some(landmarks) = landmarks {
        let distance = landmarks.pinch_distance();
        web_sys::console::log_1(
            &format!(
                "[SUCCESS] Closed pinch detected - Distance: {:.2}px, Thumb: ({:.2}, {:.2}), Index: ({:.2}, {:.2})",
                distance, landmarks.thumb_tip.x, landmarks.thumb_tip.y, landmarks.index_tip.x, landmarks.index_tip.y
            )
            .into(),
        );

        // Closed pinch should have very small distance
        assert!(
            distance < 20.0,
            "Closed pinch should have distance < 20px, got {:.2}px",
            distance
        );
    }

    web_sys::console::log_1(&"[SUCCESS] Real image closed pinch test passed ✓".into());
}

#[wasm_bindgen_test]
async fn test_real_images_distance_ordering() {
    use boid_wasm::hand_tracker::HandTracker;
    use std::collections::HashMap;

    web_sys::console::log_1(&"[TEST] Testing distance ordering across all real images...".into());

    // Wait for OpenCV.js to be ready
    let cv_ready = js_sys::eval("typeof cv !== 'undefined' && cv.Mat")
        .unwrap()
        .is_truthy();

    if !cv_ready {
        web_sys::console::log_1(&"[SKIP] OpenCV.js not loaded, skipping real image test".into());
        return;
    }

    let tracker = HandTracker::new().expect("Failed to create tracker");

    let test_images = [
        ("IMG_8522.jpeg", "open hand"),
        ("IMG_8527.jpeg", "wider/medium"),
        ("IMG_8528.jpeg", "closed pinch"),
    ];

    let mut distances: HashMap<&str, f32> = HashMap::new();

    for (filename, description) in test_images.iter() {
        web_sys::console::log_1(&format!("[TEST] Processing {}...", filename).into());

        let image_url = format!("/tests/test_images/{}", filename);
        let image_data = match load_image_data(&image_url).await {
            Ok(data) => data,
            Err(e) => {
                web_sys::console::log_1(
                    &format!("[SKIP] Failed to load image {}: {:?}", filename, e).into(),
                );
                continue;
            }
        };

        let result = tracker.process_frame(&image_data);
        assert!(result.is_ok(), "Processing {} should not error", filename);

        let landmarks = result.unwrap();
        assert!(
            landmarks.is_some(),
            "Hand should be detected in {} ({})",
            filename,
            description
        );

        if let Some(landmarks) = landmarks {
            let distance = landmarks.pinch_distance();
            distances.insert(filename, distance);
            web_sys::console::log_1(
                &format!("[INFO] {} - Distance: {:.2}px", description, distance).into(),
            );
        }
    }

    // Verify all images were processed
    if distances.len() != 3 {
        web_sys::console::log_1(
            &format!(
                "[SKIP] Could not load all images, got {} out of 3",
                distances.len()
            )
            .into(),
        );
        return;
    }

    let open_distance = *distances.get("IMG_8522.jpeg").unwrap();
    let wider_distance = *distances.get("IMG_8527.jpeg").unwrap();
    let closed_distance = *distances.get("IMG_8528.jpeg").unwrap();

    web_sys::console::log_1(
        &format!(
            "[VERIFY] Distances - Open: {:.2}px, Wider: {:.2}px, Closed: {:.2}px",
            open_distance, wider_distance, closed_distance
        )
        .into(),
    );

    // Verify distance ordering: closed < open < wider
    assert!(
        closed_distance < open_distance,
        "Closed ({:.2}px) should be < Open ({:.2}px)",
        closed_distance,
        open_distance
    );
    assert!(
        closed_distance < wider_distance,
        "Closed ({:.2}px) should be < Wider ({:.2}px)",
        closed_distance,
        wider_distance
    );
    assert!(
        open_distance < wider_distance,
        "Open ({:.2}px) should be < Wider ({:.2}px)",
        open_distance,
        wider_distance
    );

    web_sys::console::log_1(
        &"[SUCCESS] Distance ordering validated: closed < open < wider ✓".into(),
    );
}
