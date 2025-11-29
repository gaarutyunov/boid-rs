#![cfg_attr(not(feature = "std"), no_std)]

//! Hand detection library for boid simulation
//! Provides skin color-based hand tracking with fingertip detection
//! Compatible with both WASM (no_std) and native (with OpenCV) environments

use boid_shared::HandLandmarks;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// RGB color value
#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// HSV color value
#[derive(Debug, Clone, Copy)]
pub struct Hsv {
    pub h: f32, // 0-360
    pub s: f32, // 0-100
    pub v: f32, // 0-100
}

impl Rgb {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Convert RGB to HSV color space
    pub fn to_hsv(&self) -> Hsv {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;

        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;

        // Calculate hue
        let h = if delta == 0.0 {
            0.0
        } else if max == r {
            60.0 * (((g - b) / delta) % 6.0)
        } else if max == g {
            60.0 * (((b - r) / delta) + 2.0)
        } else {
            60.0 * (((r - g) / delta) + 4.0)
        };

        let h = if h < 0.0 { h + 360.0 } else { h };

        // Calculate saturation
        let s = if max == 0.0 { 0.0 } else { delta / max * 100.0 };

        // Calculate value
        let v = max * 100.0;

        Hsv { h, s, v }
    }

    /// Check if this color is likely skin tone
    pub fn is_skin_color(&self) -> bool {
        let hsv = self.to_hsv();

        // Skin color in HSV space
        // Hue: 0-50 (reddish/orange/yellow tones to accommodate different skin tones)
        // Saturation: 15-90 (allow for lighter skin tones with lower saturation)
        // Value: 25-95 (avoid very dark or very bright pixels)
        hsv.h <= 50.0 && hsv.s >= 15.0 && hsv.s <= 90.0 && hsv.v >= 25.0 && hsv.v <= 95.0
    }
}

/// A 2D point in image coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        let dx = (self.x as i32 - other.x as i32) as f32;
        let dy = (self.y as i32 - other.y as i32) as f32;
        (dx * dx + dy * dy).sqrt()
    }
}

/// Hand detector using skin color detection
pub struct HandDetector {
    min_skin_pixels: usize,
    grouping_threshold: usize,
}

impl HandDetector {
    pub fn new() -> Self {
        Self {
            min_skin_pixels: 2000,
            grouping_threshold: 30,
        }
    }

    pub fn with_min_skin_pixels(mut self, min_pixels: usize) -> Self {
        self.min_skin_pixels = min_pixels;
        self
    }

    pub fn with_grouping_threshold(mut self, threshold: usize) -> Self {
        self.grouping_threshold = threshold;
        self
    }

    /// Process an image and detect hand landmarks
    /// Image data is expected to be in RGBA format (4 bytes per pixel)
    pub fn process_rgba_image(
        &self,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> Option<HandLandmarks> {
        if data.len() < width * height * 4 {
            return None;
        }

        // Find all skin-colored pixels
        let mut skin_pixels = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 4;
                let rgb = Rgb::new(data[idx], data[idx + 1], data[idx + 2]);

                if rgb.is_skin_color() {
                    skin_pixels.push(Point::new(x, y));
                }
            }
        }

        if skin_pixels.len() < self.min_skin_pixels {
            return None;
        }

        // Find bounding box of skin region
        let min_x = skin_pixels.iter().map(|p| p.x).min()?;
        let max_x = skin_pixels.iter().map(|p| p.x).max()?;
        let min_y = skin_pixels.iter().map(|p| p.y).min()?;
        let max_y = skin_pixels.iter().map(|p| p.y).max()?;

        if max_x <= min_x || max_y <= min_y {
            return None;
        }

        // Find fingertip candidates in the top third of the hand region
        let top_threshold = min_y + (max_y - min_y) / 3;
        let mut top_points: Vec<Point> = skin_pixels
            .iter()
            .filter(|p| p.y < top_threshold)
            .copied()
            .collect();

        if top_points.len() < 2 {
            return None;
        }

        // Sort by y-coordinate (topmost first)
        top_points.sort_by_key(|p| p.y);

        // Group nearby points and find cluster centroids
        let mut finger_candidates: Vec<Point> = Vec::new();

        for point in top_points.iter().take(100) {
            let mut found_group = false;

            for candidate in finger_candidates.iter_mut() {
                if point.distance_to(candidate) < self.grouping_threshold as f32 {
                    // Average the positions
                    candidate.x = (candidate.x + point.x) / 2;
                    candidate.y = (candidate.y + point.y) / 2;
                    found_group = true;
                    break;
                }
            }

            if !found_group {
                finger_candidates.push(*point);
            }

            if finger_candidates.len() >= 5 {
                break;
            }
        }

        if finger_candidates.len() < 2 {
            return None;
        }

        // Sort by x-coordinate (leftmost first)
        finger_candidates.sort_by_key(|p| p.x);

        // Take leftmost two points as thumb and index
        let thumb = finger_candidates[0];
        let index = finger_candidates[1];

        Some(HandLandmarks::new(
            boid_shared::Position::new(thumb.x as f32, thumb.y as f32),
            boid_shared::Position::new(index.x as f32, index.y as f32),
        ))
    }

    /// Process BGR image data (OpenCV format)
    pub fn process_bgr_image(
        &self,
        width: usize,
        height: usize,
        data: &[u8],
    ) -> Option<HandLandmarks> {
        if data.len() < width * height * 3 {
            return None;
        }

        // Find all skin-colored pixels
        let mut skin_pixels = Vec::new();

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) * 3;
                // BGR format: B, G, R
                let rgb = Rgb::new(data[idx + 2], data[idx + 1], data[idx]);

                if rgb.is_skin_color() {
                    skin_pixels.push(Point::new(x, y));
                }
            }
        }

        if skin_pixels.len() < self.min_skin_pixels {
            return None;
        }

        // Find bounding box of skin region
        let min_x = skin_pixels.iter().map(|p| p.x).min()?;
        let max_x = skin_pixels.iter().map(|p| p.x).max()?;
        let min_y = skin_pixels.iter().map(|p| p.y).min()?;
        let max_y = skin_pixels.iter().map(|p| p.y).max()?;

        if max_x <= min_x || max_y <= min_y {
            return None;
        }

        // Find fingertip candidates in the top third of the hand region
        let top_threshold = min_y + (max_y - min_y) / 3;
        let mut top_points: Vec<Point> = skin_pixels
            .iter()
            .filter(|p| p.y < top_threshold)
            .copied()
            .collect();

        if top_points.len() < 2 {
            return None;
        }

        // Sort by y-coordinate (topmost first)
        top_points.sort_by_key(|p| p.y);

        // Group nearby points and find cluster centroids
        let mut finger_candidates: Vec<Point> = Vec::new();

        for point in top_points.iter().take(100) {
            let mut found_group = false;

            for candidate in finger_candidates.iter_mut() {
                if point.distance_to(candidate) < self.grouping_threshold as f32 {
                    // Average the positions
                    candidate.x = (candidate.x + point.x) / 2;
                    candidate.y = (candidate.y + point.y) / 2;
                    found_group = true;
                    break;
                }
            }

            if !found_group {
                finger_candidates.push(*point);
            }

            if finger_candidates.len() >= 5 {
                break;
            }
        }

        if finger_candidates.len() < 2 {
            return None;
        }

        // Sort by x-coordinate (leftmost first)
        finger_candidates.sort_by_key(|p| p.x);

        // Take leftmost two points as thumb and index
        let thumb = finger_candidates[0];
        let index = finger_candidates[1];

        Some(HandLandmarks::new(
            boid_shared::Position::new(thumb.x as f32, thumb.y as f32),
            boid_shared::Position::new(index.x as f32, index.y as f32),
        ))
    }
}

impl Default for HandDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_rgb_to_hsv() {
        let red = Rgb::new(255, 0, 0);
        let hsv = red.to_hsv();
        assert!((hsv.h - 0.0).abs() < 1.0);
        assert!((hsv.s - 100.0).abs() < 1.0);
        assert!((hsv.v - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_skin_color_detection() {
        // Typical skin tone
        let skin = Rgb::new(180, 150, 120);
        assert!(skin.is_skin_color());

        // Blue color (not skin)
        let blue = Rgb::new(50, 50, 200);
        assert!(!blue.is_skin_color());

        // Green color (not skin)
        let green = Rgb::new(50, 200, 50);
        assert!(!green.is_skin_color());
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0, 0);
        let p2 = Point::new(3, 4);
        assert!((p1.distance_to(&p2) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_hand_detector_no_skin() {
        let detector = HandDetector::new();

        // Create a 10x10 blue image (RGBA)
        let mut data = vec![0u8; 10 * 10 * 4];
        for i in 0..(10 * 10) {
            data[i * 4] = 50; // R
            data[i * 4 + 1] = 50; // G
            data[i * 4 + 2] = 200; // B
            data[i * 4 + 3] = 255; // A
        }

        let result = detector.process_rgba_image(10, 10, &data);
        assert!(result.is_none());
    }

    #[test]
    fn test_hand_detector_with_skin_pixels() {
        let detector = HandDetector::new().with_min_skin_pixels(500);

        // Create a 200x200 image with skin-colored region (larger for easier detection)
        let mut data = vec![0u8; 200 * 200 * 4];

        // Fill with white background
        for i in 0..(200 * 200) {
            data[i * 4] = 255;
            data[i * 4 + 1] = 255;
            data[i * 4 + 2] = 255;
            data[i * 4 + 3] = 255;
        }

        // Add skin-colored palm region in the center-bottom
        for y in 80..180 {
            for x in 60..140 {
                let idx = (y * 200 + x) * 4;
                data[idx] = 180; // R
                data[idx + 1] = 150; // G
                data[idx + 2] = 120; // B
                data[idx + 3] = 255; // A
            }
        }

        // Add two finger-like protrusions at the top (larger)
        for y in 40..80 {
            for x in 70..80 {
                // Left finger (thumb)
                let idx = (y * 200 + x) * 4;
                data[idx] = 180;
                data[idx + 1] = 150;
                data[idx + 2] = 120;
                data[idx + 3] = 255;
            }
            for x in 120..130 {
                // Right finger (index)
                let idx = (y * 200 + x) * 4;
                data[idx] = 180;
                data[idx + 1] = 150;
                data[idx + 2] = 120;
                data[idx + 3] = 255;
            }
        }

        let result = detector.process_rgba_image(200, 200, &data);
        assert!(
            result.is_some(),
            "Hand should be detected in synthetic image"
        );

        let landmarks = result.unwrap();
        // Verify that thumb is to the left of index
        assert!(
            landmarks.thumb_tip.x < landmarks.index_tip.x,
            "Thumb should be to the left of index finger"
        );
    }
}
