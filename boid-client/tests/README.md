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

Contains three main test categories:

1. **Mock Server Tests** (`test_mock_server_receives_position_updates`)
   - Validates that the HTTP client can send position updates to the mock server
   - Verifies JSON serialization and deserialization
   - Tests handling of both detected positions and "no hand" scenarios

2. **Synthetic Image Tests** (`test_create_synthetic_pinch_images`)
   - Creates hand gesture images with different pinch distances (close, medium, wide)
   - Validates image dimensions and format
   - Saves images to `tests/test_images/` for manual inspection

3. **Hand Tracker Tests** (`test_hand_tracker_with_synthetic_pinch_images`)
   - Tests the `HandTracker` with synthetic pinch gestures
   - Validates that pinch distance detection works correctly
   - Ensures close pinches have small distances and wide pinches have larger distances

4. **Full Integration Tests** (`test_client_integration_with_mock_server`)
   - End-to-end test combining all components
   - Processes multiple synthetic images through the hand tracker
   - Sends detected positions to the mock server
   - Verifies that HTTP requests are received correctly

## Synthetic Image Generation

The tests create hand-like shapes programmatically using OpenCV drawing functions:

- **Palm**: Large ellipse in skin tone color
- **Fingers**: Two circles representing thumb and index fingertips
- **Connections**: Lines connecting fingertips to palm

Pinch distance is controlled by adjusting the separation between the two fingertip circles.

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

The integration tests use **synthetic images** instead of requiring external datasets because:

1. **Reproducibility**: Tests produce consistent results across environments
2. **No External Dependencies**: No need to download large datasets
3. **Parameterized Testing**: Easy to generate images with specific characteristics
4. **Fast Execution**: Small, programmatic images are quick to generate
5. **CI/CD Friendly**: Works in automated build environments

However, the test infrastructure supports real images for more comprehensive validation when needed.

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

### Tests fail to detect hand in synthetic images

This is expected behavior for the simplified hand tracker, which uses skin color detection and contour analysis. The synthetic images use skin tone colors and may not perfectly match real hand detection. Adjust:

- `min_contour_area` in `HandTracker::new()`
- Skin color HSV range in `process_frame()`
- Finger detection logic in `extract_hand_landmarks()`

## Future Improvements

- [ ] Add tests with real hand gesture images from datasets
- [ ] Test with various lighting conditions (synthetic brightness/contrast variations)
- [ ] Add performance benchmarks for hand tracking
- [ ] Test multi-hand scenarios
- [ ] Add tests for gesture sequence recognition
- [ ] Integration with MJPEG video stream simulation

## Related Documentation

- Project overview: `/CLAUDE.md`
- Hand tracker implementation: `../src/hand_tracker.rs`
- Shared types: `../../boid-shared/src/lib.rs`
