/**
 * Hand detection using OpenCV.js
 * This module implements skin color-based hand detection with fingertip tracking
 * Similar to the algorithm used in boid-client
 */

export function processHandDetection(imageData, minArea) {
    // Check if OpenCV.js is loaded
    if (typeof cv === 'undefined') {
        console.error('OpenCV.js not loaded');
        return null;
    }

    try {
        // Create Mat from ImageData (RGBA format)
        const src = cv.matFromImageData(imageData);

        // Convert RGBA to BGR
        const bgr = new cv.Mat();
        cv.cvtColor(src, bgr, cv.COLOR_RGBA2BGR, 0);

        // Convert BGR to HSV for better skin color detection
        const hsv = new cv.Mat();
        cv.cvtColor(bgr, hsv, cv.COLOR_BGR2HSV, 0);

        // Define skin color range in HSV
        // H: 0-20 (reddish/orange tones)
        // S: 20-255 (decent saturation)
        // V: 70-255 (not too dark)
        const lowerSkin = new cv.Mat(hsv.rows, hsv.cols, hsv.type(), [0, 20, 70, 0]);
        const upperSkin = new cv.Mat(hsv.rows, hsv.cols, hsv.type(), [20, 255, 255, 255]);

        // Create skin color mask
        const mask = new cv.Mat();
        cv.inRange(hsv, lowerSkin, upperSkin, mask);

        // Apply morphological operations to remove noise
        const kernel = cv.getStructuringElement(cv.MORPH_ELLIPSE, new cv.Size(5, 5));

        // Close operation (dilate then erode) - fills small holes
        cv.morphologyEx(mask, mask, cv.MORPH_CLOSE, kernel, new cv.Point(-1, -1), 2);

        // Open operation (erode then dilate) - removes small noise
        cv.morphologyEx(mask, mask, cv.MORPH_OPEN, kernel, new cv.Point(-1, -1), 2);

        // Apply Gaussian blur to smooth the mask
        cv.GaussianBlur(mask, mask, new cv.Size(5, 5), 0, 0, cv.BORDER_DEFAULT);

        // Find contours
        const contours = new cv.MatVector();
        const hierarchy = new cv.Mat();
        cv.findContours(mask, contours, hierarchy, cv.RETR_EXTERNAL, cv.CHAIN_APPROX_SIMPLE);

        // Find the largest contour (assumed to be the hand)
        let maxArea = 0;
        let maxContourIndex = -1;

        for (let i = 0; i < contours.size(); i++) {
            const contour = contours.get(i);
            const area = cv.contourArea(contour, false);
            if (area > maxArea) {
                maxArea = area;
                maxContourIndex = i;
            }
        }

        let result = null;

        // If we found a large enough contour, extract hand landmarks
        if (maxContourIndex >= 0 && maxArea > minArea) {
            const contour = contours.get(maxContourIndex);
            result = extractHandLandmarks(contour);
        }

        // Cleanup
        src.delete();
        bgr.delete();
        hsv.delete();
        lowerSkin.delete();
        upperSkin.delete();
        mask.delete();
        kernel.delete();
        contours.delete();
        hierarchy.delete();

        return result;
    } catch (error) {
        console.error('Hand detection error:', error);
        return null;
    }
}

/**
 * Extract thumb and index finger positions from hand contour
 */
function extractHandLandmarks(contour) {
    try {
        // Find convex hull
        const hull = new cv.Mat();
        cv.convexHull(contour, hull, false, true);

        // Find convexity defects to identify fingers
        const hullIndices = new cv.Mat();
        cv.convexHull(contour, hullIndices, false, false);

        const defects = new cv.Mat();
        if (hullIndices.rows > 3) {
            cv.convexityDefects(contour, hullIndices, defects);
        }

        // Get hull points
        const hullPoints = [];
        for (let i = 0; i < hull.rows; i++) {
            const point = {
                x: hull.intPtr(i, 0)[0],
                y: hull.intPtr(i, 0)[1]
            };
            hullPoints.push(point);
        }

        // Find fingertips (topmost points in the hull)
        // Sort hull points by y-coordinate (topmost first)
        hullPoints.sort((a, b) => a.y - b.y);

        // Take top 5 points as potential fingertips
        const topPoints = hullPoints.slice(0, Math.min(5, hullPoints.length));

        if (topPoints.length < 2) {
            hull.delete();
            hullIndices.delete();
            defects.delete();
            return null;
        }

        // Sort by x-coordinate to identify thumb (leftmost) and index (next)
        topPoints.sort((a, b) => a.x - b.x);

        const thumb = topPoints[0];
        const index = topPoints[1];

        // Cleanup
        hull.delete();
        hullIndices.delete();
        defects.delete();

        return {
            thumbX: thumb.x,
            thumbY: thumb.y,
            indexX: index.x,
            indexY: index.y
        };
    } catch (error) {
        console.error('Error extracting hand landmarks:', error);
        return null;
    }
}
