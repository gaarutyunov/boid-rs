#ifndef BOID_MEDIAPIPE_WRAPPER_H
#define BOID_MEDIAPIPE_WRAPPER_H

#ifdef __cplusplus
extern "C" {
#endif

// Hand landmark structure (21 landmarks per hand)
typedef struct {
    float x;
    float y;
    float z;
    float visibility;
    float presence;
} MediaPipeLandmark;

typedef struct {
    MediaPipeLandmark landmarks[21];
    int handedness; // 0 = left, 1 = right
} MediaPipeHand;

// Hand detector handle
typedef struct MediaPipeHandDetector MediaPipeHandDetector;

// Create a new hand detector
MediaPipeHandDetector* mediapipe_hand_detector_create();

// Destroy the hand detector
void mediapipe_hand_detector_destroy(MediaPipeHandDetector* detector);

// Process an image frame (BGR format)
// Returns the number of hands detected (0, 1, or 2)
int mediapipe_hand_detector_process(
    MediaPipeHandDetector* detector,
    const unsigned char* image_data,
    int width,
    int height,
    MediaPipeHand* hands,
    int max_hands
);

#ifdef __cplusplus
}
#endif

#endif // BOID_MEDIAPIPE_WRAPPER_H
