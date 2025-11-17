# Boid Client Integration Tests

This directory contains integration tests for the boid-client that simulate hand gesture recognition and HTTP communication with a mock ESP32 server.

## Overview

The integration tests validate the complete client workflow:

1. **Synthetic Image Generation**: Creates programmatic hand gesture images with various pinch distances
2. **Hand Tracking**: Processes images through the `HandTracker` to detect finger positions
3. **Mock HTTP Server**: Simulates the ESP32 server API using `wiremock`
4. **End-to-End Testing**: Verifies that detected hand positions are correctly sent to the server

## Test Structure

### `integration_test.rs`

Contains several test categories:

1. **Mock Server Tests** (`test_mock_server_receives_position_updates`)
   - Validates that the HTTP client can send position updates to the mock server
   - Verifies JSON serialization and deserialization
   - Tests handling of both detected positions and "no hand" scenarios

2. **Synthetic Image Tests** (`test_create_synthetic_pinch_images`)
   - Creates hand gesture images with different pinch distances (close, medium, wide)
   - Validates image dimensions and format
   - Saves images to `tests/test_images/` for manual inspection
   - **Note**: Informational only - validates image generation, not detection

3. **Hand Tracker Tests with Synthetic Images** (`test_hand_tracker_with_synthetic_pinch_images`)
   - Tests the `HandTracker` with synthetic pinch gestures
   - **Informational test** - synthetic images may not be detected by skin-color tracker
   - Demonstrates tracker behavior with programmatically generated images

4. **Hand Tracker Tests with Real Images** (`test_hand_tracker_with_real_pinch_images`)
   - **Primary validation test** - uses actual hand gesture photos
   - Tests with open hand, medium, and closed pinch gestures
   - Validates pinch distance calculations with real-world images
   - Logs detection results and finger positions

5. **Integration Tests with Synthetic Images** (`test_client_integration_with_mock_server_synthetic`)
   - End-to-end test with programmatically generated images
   - Tests HTTP communication flow
   - Validates that all images result in position updates being sent

6. **Integration Tests with Real Images** (`test_client_integration_with_mock_server_real_images`)
   - **Primary end-to-end validation** - uses actual hand gesture photos
   - Tests complete workflow: image loading → hand tracking → HTTP updates
   - Validates detection and position data with real-world gestures

## Real Hand Gesture Images

The primary test validation uses real hand gesture photos located in `tests/test_images/`:

- `IMG_8522.jpeg` - **Open hand** (fingers apart, largest distance)
- `IMG_8527.jpeg` - **Wider/medium** (fingers moderately separated)
- `IMG_8528.jpeg` - **Closed pinch** (fingers close together, smallest distance)

These images are actual iPhone photos (4032x3024) that provide realistic test data for the skin-color-based hand tracker.

## Synthetic Image Generation

The tests also create hand-like shapes programmatically using OpenCV drawing functions for baseline testing:

- **Palm**: Large ellipse in skin tone color
- **Fingers**: Two circles representing thumb and index fingertips
- **Connections**: Lines connecting fingertips to palm

Pinch distance is controlled by adjusting the separation between the two fingertip circles.

**Important Note**: Synthetic images use simplified shapes and colors that may not match the HSV skin-color detection criteria used by the hand tracker. These tests are **informational only** and demonstrate image generation capabilities. The real image tests provide the actual validation of hand detection functionality.

### Generated Test Images

When tests run, they create these images in `tests/test_images/`:

- `pinch_close.jpg` - Fingers 50px apart (tight pinch)
- `pinch_medium.jpg` - Fingers 100px apart (medium pinch)
- `pinch_wide.jpg` - Fingers 200px apart (wide/open hand)
- `no_hand.jpg` - Empty background (no hand detected)

## Requirements

### System Dependencies

The tests require OpenCV to be installed on the system:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y \
    libopencv-dev \
    clang \
    libclang-dev \
    pkg-config
```

**macOS:**
```bash
brew install opencv
```

### Environment Variables

If libclang is not found automatically, set:
```bash
export LIBCLANG_PATH=/usr/lib/llvm-18/lib  # Adjust version as needed
```

For OpenCV pkg-config issues:
```bash
export PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/local/lib/pkgconfig
```

## Running the Tests

### Run all integration tests:
```bash
cargo test --test integration_test
```

### Run with output:
```bash
cargo test --test integration_test -- --nocapture
```

### Run specific test:
```bash
cargo test --test integration_test test_client_integration_with_mock_server
```

### Run with environment variables:
```bash
LIBCLANG_PATH=/usr/lib/llvm-18/lib cargo test --test integration_test
```

## Using Real Hand Gesture Images

The test infrastructure is designed to be extensible. To use real hand gesture images (e.g., from a dataset):

1. Download hand gesture images to `tests/test_images/`
2. Modify the test to load images using `imgcodecs::imread`:

```rust
use opencv::imgcodecs;

let img = imgcodecs::imread("tests/test_images/real_pinch.jpg", imgcodecs::IMREAD_COLOR)?;
let result = tracker.process_frame(&img)?;
```

3. The HaGRID dataset mentioned in the search results would be a good source:
   - Repository: https://github.com/hukenovs/hagrid
   - Contains 1M+ hand gesture images including pinch gestures

## Test Design Philosophy

The integration tests use a **dual approach** combining real and synthetic images:

### Real Images (Primary Validation)
Real hand gesture photos provide the primary validation:
1. **Realistic Testing**: Actual iPhone photos with real skin tones and hand shapes
2. **Proven Detection**: Images known to work with skin-color-based hand tracker
3. **Real-World Validation**: Tests against actual use-case scenarios
4. **Included in Repository**: Images stored in `tests/test_images/`

### Synthetic Images (Baseline Testing)
Programmatically generated images for baseline functionality:
1. **Reproducibility**: Generate consistent test images on-demand
2. **Parameterized Testing**: Control exact pinch distances and positions
3. **Fast Execution**: Quick to generate without file I/O
4. **CI/CD Friendly**: No external dependencies
5. **Informational**: Demonstrates capabilities but may not pass detection

**Note**: Synthetic images use simplified shapes that may not match HSV skin-color detection criteria. Real image tests provide the actual validation of hand detection functionality.

## Troubleshooting

### OpenCV not found

```
Error: Failed to find installed OpenCV package
```

**Solution**: Install OpenCV development packages (see Requirements above)

### libclang not found

```
Error: couldn't find any valid shared libraries matching: ['libclang.so', 'libclang-*.so']
```

**Solution**:
1. Find libclang: `find /usr/lib -name "libclang*.so"`
2. Set LIBCLANG_PATH to that directory
3. Alternatively, create a symlink: `ln -s /usr/lib/llvm-XX/lib/libclang.so.1 /usr/local/lib/libclang.so`

### Synthetic images are not detected by hand tracker

**This is expected behavior.** Synthetic images use simplified geometric shapes and may not match the HSV skin-color detection criteria. The tests are designed to handle this:

- `test_hand_tracker_with_synthetic_pinch_images` - **Informational test**, logs results without failing
- `test_hand_tracker_with_real_pinch_images` - **Primary validation test**, uses real photos

If you need synthetic images to be detected, you would need to adjust the hand tracker's detection parameters or improve the synthetic image generation to better match real skin tones and textures. However, the real image tests provide comprehensive validation, making this unnecessary.

## Future Improvements

- [x] Add tests with real hand gesture images from datasets (completed - using iPhone photos)
- [ ] Add more real gesture images with various hand positions and orientations
- [ ] Test with various lighting conditions
- [ ] Add performance benchmarks for hand tracking
- [ ] Test multi-hand scenarios
- [ ] Add tests for gesture sequence recognition
- [ ] Integration with MJPEG video stream simulation
- [ ] Improve synthetic image generation to match real skin tones (if needed)

## Related Documentation

- Project overview: `/CLAUDE.md`
- Hand tracker implementation: `../src/hand_tracker.rs`
- Shared types: `../../boid-shared/src/lib.rs`
