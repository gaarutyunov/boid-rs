use anyhow::Result;
use boid_shared::{HandLandmarks, Position};

// Include the generated bindings
#[allow(non_upper_case_globals)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[allow(dead_code)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use bindings::*;

pub struct HandDetector {
    detector: *mut MediaPipeHandDetector,
}

impl HandDetector {
    /// Create a new MediaPipe hand detector
    pub fn new() -> Result<Self> {
        let detector = unsafe { mediapipe_hand_detector_create() };
        if detector.is_null() {
            anyhow::bail!("Failed to create MediaPipe hand detector");
        }
        Ok(Self { detector })
    }

    /// Process a BGR image frame and detect hands
    /// Returns HandLandmarks if at least one hand is detected
    pub fn process_frame(
        &mut self,
        image_data: &[u8],
        width: i32,
        height: i32,
    ) -> Result<Option<HandLandmarks>> {
        let mut hands = [MediaPipeHand {
            landmarks: [MediaPipeLandmark {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                visibility: 0.0,
                presence: 0.0,
            }; 21],
            handedness: 0,
        }; 2];

        let num_hands = unsafe {
            mediapipe_hand_detector_process(
                self.detector,
                image_data.as_ptr(),
                width,
                height,
                hands.as_mut_ptr(),
                2,
            )
        };

        if num_hands > 0 {
            // Get the first hand detected
            let hand = &hands[0];

            // MediaPipe hand landmarks:
            // 4 = Thumb tip
            // 8 = Index finger tip
            let thumb_tip = &hand.landmarks[4];
            let index_tip = &hand.landmarks[8];

            // Convert normalized coordinates to pixel coordinates
            let thumb_pos = Position::new(thumb_tip.x * width as f32, thumb_tip.y * height as f32);
            let index_pos = Position::new(index_tip.x * width as f32, index_tip.y * height as f32);

            Ok(Some(HandLandmarks::new(thumb_pos, index_pos)))
        } else {
            Ok(None)
        }
    }
}

impl Drop for HandDetector {
    fn drop(&mut self) {
        if !self.detector.is_null() {
            unsafe {
                mediapipe_hand_detector_destroy(self.detector);
            }
        }
    }
}

// Ensure HandDetector is Send + Sync for use across threads
unsafe impl Send for HandDetector {}
unsafe impl Sync for HandDetector {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        // This test will only pass when MediaPipe is properly installed
        // Skip if MEDIAPIPE_DIR is not set
        if std::env::var("MEDIAPIPE_DIR").is_err() {
            eprintln!("Skipping test: MEDIAPIPE_DIR not set");
            return;
        }

        let detector = HandDetector::new();
        assert!(detector.is_ok());
    }
}
