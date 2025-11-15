use anyhow::Result;
use boid_shared::{HandLandmarks, Position};
use opencv::{
    core::{self, Mat, Point, Scalar, Size, Vector, BORDER_DEFAULT, CV_8UC1},
    imgproc,
    prelude::*,
};

pub struct HandTracker {
    // Store previous frame for motion detection if needed
    min_contour_area: f64,
}

impl HandTracker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            min_contour_area: 5000.0, // Minimum area to consider as a hand
        })
    }

    /// Process a frame and detect hand landmarks
    /// Returns HandLandmarks if a hand is detected
    pub fn process_frame(&mut self, frame: &Mat) -> Result<Option<HandLandmarks>> {
        // Convert to HSV for better skin color detection
        let mut hsv = Mat::default();
        imgproc::cvt_color(frame, &mut hsv, imgproc::COLOR_BGR2HSV, 0)?;

        // Define skin color range in HSV
        // These values work well for various skin tones
        let lower_skin = Scalar::new(0.0, 20.0, 70.0, 0.0);
        let upper_skin = Scalar::new(20.0, 255.0, 255.0, 0.0);

        // Create mask for skin color
        let mut mask = Mat::default();
        core::in_range(&hsv, &lower_skin, &upper_skin, &mut mask)?;

        // Apply morphological operations to remove noise
        let kernel = imgproc::get_structuring_element(
            imgproc::MORPH_ELLIPSE,
            Size::new(5, 5),
            Point::new(-1, -1),
        )?;
        imgproc::morphology_ex(
            &mask,
            &mut mask,
            imgproc::MORPH_CLOSE,
            &kernel,
            Point::new(-1, -1),
            2,
            BORDER_DEFAULT,
            core::Scalar::default(),
        )?;
        imgproc::morphology_ex(
            &mask,
            &mut mask,
            imgproc::MORPH_OPEN,
            &kernel,
            Point::new(-1, -1),
            2,
            BORDER_DEFAULT,
            core::Scalar::default(),
        )?;

        // Apply Gaussian blur to smooth the mask
        imgproc::gaussian_blur(
            &mask,
            &mut mask,
            Size::new(5, 5),
            0.0,
            0.0,
            BORDER_DEFAULT,
        )?;

        // Find contours
        let mut contours = Vector::<Vector<Point>>::new();
        imgproc::find_contours(
            &mask,
            &mut contours,
            imgproc::RETR_EXTERNAL,
            imgproc::CHAIN_APPROX_SIMPLE,
            Point::new(0, 0),
        )?;

        // Find the largest contour (assumed to be the hand)
        let mut max_area = 0.0;
        let mut max_contour_idx = None;

        for (idx, contour) in contours.iter().enumerate() {
            let area = imgproc::contour_area(&contour, false)?;
            if area > max_area {
                max_area = area;
                max_contour_idx = Some(idx);
            }
        }

        // If we found a large enough contour, extract hand landmarks
        if let Some(idx) = max_contour_idx {
            if max_area > self.min_contour_area {
                let contour = &contours.get(idx)?;
                return self.extract_hand_landmarks(contour, frame);
            }
        }

        Ok(None)
    }

    /// Extract thumb and index finger positions from hand contour
    /// This is a simplified approach using convexity defects
    fn extract_hand_landmarks(
        &self,
        contour: &Vector<Point>,
        frame: &Mat,
    ) -> Result<Option<HandLandmarks>> {
        // Find convex hull
        let mut hull_indices = Vector::<i32>::new();
        imgproc::convex_hull_idx(contour, &mut hull_indices, false, false)?;

        if hull_indices.len() < 3 {
            return Ok(None);
        }

        // Find convexity defects
        let mut defects = Vector::<core::Vec4i>::new();
        if let Err(_) = imgproc::convexity_defects(contour, &hull_indices, &mut defects) {
            // If we can't find defects, fall back to centroid and topmost point
            return self.simple_landmark_detection(contour, frame);
        }

        // Find fingertips (convex hull points that are far from palm)
        let mut fingertips = Vec::new();

        for i in 0..hull_indices.len() {
            let idx = hull_indices.get(i)? as usize;
            if idx < contour.len() {
                let point = contour.get(idx)?;
                fingertips.push(point);
            }
        }

        if fingertips.len() < 2 {
            return self.simple_landmark_detection(contour, frame);
        }

        // Sort fingertips by y-coordinate (topmost points are likely fingertips)
        fingertips.sort_by(|a, b| a.y.cmp(&b.y));

        // Take top 2 points as finger tips (thumb and index)
        // For left/right distinction, use x-coordinate
        let mut top_points = fingertips.iter().take(5).cloned().collect::<Vec<_>>();
        top_points.sort_by(|a, b| a.x.cmp(&b.x));

        if top_points.len() >= 2 {
            // Assume leftmost is thumb, next is index (works for right hand)
            // For left hand, this would be reversed, but we'll keep it simple
            let thumb_tip = Position::new(top_points[0].x as f32, top_points[0].y as f32);
            let index_tip = Position::new(top_points[1].x as f32, top_points[1].y as f32);

            return Ok(Some(HandLandmarks::new(thumb_tip, index_tip)));
        }

        Ok(None)
    }

    /// Simple fallback: use centroid and topmost point
    fn simple_landmark_detection(
        &self,
        contour: &Vector<Point>,
        _frame: &Mat,
    ) -> Result<Option<HandLandmarks>> {
        // Find moments to calculate centroid
        let moments = imgproc::moments(contour, false)?;

        if moments.m00 == 0.0 {
            return Ok(None);
        }

        let cx = (moments.m10 / moments.m00) as f32;
        let cy = (moments.m01 / moments.m00) as f32;

        // Find topmost point in the contour
        let mut topmost = contour.get(0)?;
        for i in 1..contour.len() {
            let point = contour.get(i)?;
            if point.y < topmost.y {
                topmost = point;
            }
        }

        // Find another high point that's not too close to topmost
        let mut second_top = contour.get(0)?;
        let mut min_y = i32::MAX;
        for i in 0..contour.len() {
            let point = contour.get(i)?;
            let distance = ((point.x - topmost.x).pow(2) + (point.y - topmost.y).pow(2)) as f32;
            if point.y < min_y && distance > 30.0 {
                min_y = point.y;
                second_top = point;
            }
        }

        let thumb_tip = Position::new(topmost.x as f32, topmost.y as f32);
        let index_tip = Position::new(second_top.x as f32, second_top.y as f32);

        Ok(Some(HandLandmarks::new(thumb_tip, index_tip)))
    }
}
