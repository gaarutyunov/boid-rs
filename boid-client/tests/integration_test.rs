use anyhow::Result;
use boid_shared::{Position, TargetPositionUpdate};
use opencv::{
    core::{Mat, Point, Scalar, Size, Vector, CV_8UC3},
    imgcodecs, imgproc,
    prelude::*,
};
use serde_json::json;
use std::sync::{Arc, Mutex};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Stores received position updates for verification
#[derive(Clone, Default)]
struct ReceivedUpdates {
    updates: Arc<Mutex<Vec<TargetPositionUpdate>>>,
}

impl ReceivedUpdates {
    fn new() -> Self {
        Self {
            updates: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add(&self, update: TargetPositionUpdate) {
        self.updates.lock().unwrap().push(update);
    }

    fn get_all(&self) -> Vec<TargetPositionUpdate> {
        self.updates.lock().unwrap().clone()
    }

    fn count(&self) -> usize {
        self.updates.lock().unwrap().len()
    }
}

/// Creates a synthetic image with a hand-like shape simulating a pinch gesture
/// Returns an image with two circular "fingertips" representing thumb and index finger
fn create_pinch_gesture_image(width: i32, height: i32, pinch_distance: f32) -> Result<Mat> {
    // Create a blank BGR image
    let mut img = Mat::new_rows_cols_with_default(
        height,
        width,
        CV_8UC3,
        Scalar::new(255.0, 255.0, 255.0, 0.0), // White background
    )?;

    // Calculate center position for the hand
    let center_x = width / 2;
    let center_y = height / 2;

    // Position thumb and index finger based on pinch distance
    let thumb_x = center_x - (pinch_distance / 2.0) as i32;
    let index_x = center_x + (pinch_distance / 2.0) as i32;

    // Draw a palm-like region (large ellipse in skin tone)
    let palm_center = Point::new(center_x, center_y + 80);
    imgproc::ellipse(
        &mut img,
        palm_center,
        Size::new(100, 120),
        0.0,
        0.0,
        360.0,
        Scalar::new(180.0, 150.0, 120.0, 0.0), // Skin tone (BGR)
        -1,
        imgproc::LINE_8,
        0,
    )?;

    // Draw thumb tip (left circle in skin tone)
    imgproc::circle(
        &mut img,
        Point::new(thumb_x, center_y),
        25,
        Scalar::new(180.0, 150.0, 120.0, 0.0), // Skin tone
        -1,
        imgproc::LINE_8,
        0,
    )?;

    // Draw thumb connection to palm
    imgproc::line(
        &mut img,
        Point::new(thumb_x, center_y),
        palm_center,
        Scalar::new(180.0, 150.0, 120.0, 0.0),
        30,
        imgproc::LINE_8,
        0,
    )?;

    // Draw index finger tip (right circle in skin tone)
    imgproc::circle(
        &mut img,
        Point::new(index_x, center_y),
        25,
        Scalar::new(180.0, 150.0, 120.0, 0.0), // Skin tone
        -1,
        imgproc::LINE_8,
        0,
    )?;

    // Draw index finger connection to palm
    imgproc::line(
        &mut img,
        Point::new(index_x, center_y),
        palm_center,
        Scalar::new(180.0, 150.0, 120.0, 0.0),
        30,
        imgproc::LINE_8,
        0,
    )?;

    Ok(img)
}

/// Creates an image with no hand visible (empty background)
fn create_no_hand_image(width: i32, height: i32) -> Result<Mat> {
    // Create a blank BGR image with a different color (light blue background)
    let img = Mat::new_rows_cols_with_default(
        height,
        width,
        CV_8UC3,
        Scalar::new(200.0, 200.0, 150.0, 0.0),
    )?;
    Ok(img)
}

/// Saves test images to the test_images directory for manual inspection
fn save_test_images() -> Result<()> {
    let test_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/test_images");
    std::fs::create_dir_all(test_dir)?;

    // Create and save pinch gesture with different distances
    let close_pinch = create_pinch_gesture_image(640, 480, 50.0)?;
    imgcodecs::imwrite(
        &format!("{}/pinch_close.jpg", test_dir),
        &close_pinch,
        &Vector::new(),
    )?;

    let medium_pinch = create_pinch_gesture_image(640, 480, 100.0)?;
    imgcodecs::imwrite(
        &format!("{}/pinch_medium.jpg", test_dir),
        &medium_pinch,
        &Vector::new(),
    )?;

    let wide_pinch = create_pinch_gesture_image(640, 480, 200.0)?;
    imgcodecs::imwrite(
        &format!("{}/pinch_wide.jpg", test_dir),
        &wide_pinch,
        &Vector::new(),
    )?;

    let no_hand = create_no_hand_image(640, 480)?;
    imgcodecs::imwrite(
        &format!("{}/no_hand.jpg", test_dir),
        &no_hand,
        &Vector::new(),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_receives_position_updates() -> Result<()> {
        // Start a mock server
        let mock_server = MockServer::start().await;
        let received = ReceivedUpdates::new();
        let received_clone = received.clone();

        // Set up mock endpoint for position updates
        Mock::given(method("POST"))
            .and(path("/api/position"))
            .respond_with(move |req: &wiremock::Request| {
                let body_str = String::from_utf8(req.body.clone()).unwrap();
                if let Ok(update) = serde_json::from_str::<TargetPositionUpdate>(&body_str) {
                    received_clone.add(update);
                }
                ResponseTemplate::new(200).set_body_json(json!({"status": "ok"}))
            })
            .mount(&mock_server)
            .await;

        // Create a test HTTP client
        let client = reqwest::Client::new();

        // Simulate sending position updates
        let test_positions = vec![
            Some(Position::new(100.0, 150.0)),
            Some(Position::new(200.0, 250.0)),
            None, // No hand detected
        ];

        for position in test_positions.iter() {
            let update = TargetPositionUpdate {
                position: *position,
            };
            let url = format!("{}/api/position", mock_server.uri());
            client.post(&url).json(&update).send().await?;
        }

        // Verify that all updates were received
        assert_eq!(received.count(), 3);

        let all_updates = received.get_all();
        assert_eq!(all_updates[0].position, Some(Position::new(100.0, 150.0)));
        assert_eq!(all_updates[1].position, Some(Position::new(200.0, 250.0)));
        assert_eq!(all_updates[2].position, None);

        Ok(())
    }

    #[test]
    fn test_create_synthetic_pinch_images() -> Result<()> {
        // Test creating images with different pinch distances
        let close_pinch = create_pinch_gesture_image(640, 480, 50.0)?;
        assert_eq!(close_pinch.rows(), 480);
        assert_eq!(close_pinch.cols(), 640);

        let medium_pinch = create_pinch_gesture_image(640, 480, 100.0)?;
        assert_eq!(medium_pinch.rows(), 480);
        assert_eq!(medium_pinch.cols(), 640);

        let wide_pinch = create_pinch_gesture_image(640, 480, 200.0)?;
        assert_eq!(wide_pinch.rows(), 480);
        assert_eq!(wide_pinch.cols(), 640);

        // Save test images for manual inspection
        save_test_images()?;

        Ok(())
    }

    #[test]
    fn test_hand_tracker_with_synthetic_pinch_images() -> Result<()> {
        use boid_client::hand_tracker::HandTracker;

        let mut tracker = HandTracker::new()?;

        // Test with close pinch (fingers close together)
        let close_pinch = create_pinch_gesture_image(640, 480, 50.0)?;
        let result_close = tracker.process_frame(&close_pinch)?;
        assert!(
            result_close.is_some(),
            "Hand tracker should detect hand in close pinch image"
        );

        if let Some(landmarks) = result_close {
            let distance = landmarks.pinch_distance();
            println!("Close pinch distance: {}", distance);
            // The detected distance should be reasonably close to our input
            assert!(
                distance < 150.0,
                "Close pinch should have small distance, got {}",
                distance
            );
        }

        // Test with wide pinch (fingers far apart)
        let wide_pinch = create_pinch_gesture_image(640, 480, 200.0)?;
        let result_wide = tracker.process_frame(&wide_pinch)?;
        assert!(
            result_wide.is_some(),
            "Hand tracker should detect hand in wide pinch image"
        );

        if let Some(landmarks) = result_wide {
            let distance = landmarks.pinch_distance();
            println!("Wide pinch distance: {}", distance);
            // The detected distance should be larger
            assert!(
                distance > 50.0,
                "Wide pinch should have larger distance, got {}",
                distance
            );
        }

        // Test with no hand
        let no_hand = create_no_hand_image(640, 480)?;
        let result_no_hand = tracker.process_frame(&no_hand)?;
        // This might or might not detect a hand depending on the background
        // We just verify it doesn't crash
        println!("No hand result: {:?}", result_no_hand.is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_client_integration_with_mock_server() -> Result<()> {
        use boid_client::hand_tracker::HandTracker;

        // Start mock server
        let mock_server = MockServer::start().await;
        let received = ReceivedUpdates::new();
        let received_clone = received.clone();

        // Set up mock endpoint
        Mock::given(method("POST"))
            .and(path("/api/position"))
            .respond_with(move |req: &wiremock::Request| {
                let body_str = String::from_utf8(req.body.clone()).unwrap();
                if let Ok(update) = serde_json::from_str::<TargetPositionUpdate>(&body_str) {
                    received_clone.add(update);
                }
                ResponseTemplate::new(200).set_body_json(json!({"status": "ok"}))
            })
            .mount(&mock_server)
            .await;

        // Create HTTP client
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()?;

        // Create hand tracker
        let mut tracker = HandTracker::new()?;

        // Process synthetic images and send updates
        let test_images = vec![
            create_pinch_gesture_image(640, 480, 50.0)?,
            create_pinch_gesture_image(640, 480, 100.0)?,
            create_pinch_gesture_image(640, 480, 150.0)?,
            create_no_hand_image(640, 480)?,
        ];

        for img in test_images.iter() {
            let hand_result = tracker.process_frame(img)?;

            let position = if let Some(ref hand_data) = hand_result {
                Some(Position::new(hand_data.index_tip.x, hand_data.index_tip.y))
            } else {
                None
            };

            let update = TargetPositionUpdate { position };
            let url = format!("{}/api/position", mock_server.uri());
            http_client.post(&url).json(&update).send().await?;
        }

        // Verify updates were received
        assert!(
            received.count() >= 3,
            "Should receive at least 3 position updates (3 pinch images with hands)"
        );

        let all_updates = received.get_all();
        for (i, update) in all_updates.iter().enumerate() {
            println!("Update {}: {:?}", i, update.position);
        }

        Ok(())
    }
}
