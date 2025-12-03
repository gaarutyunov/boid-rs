# MediaPipe Integration Guide

This guide explains how to build and use MediaPipe for hand tracking in the boid-rs project.

## Overview

The `boid-mediapipe` crate provides Rust FFI bindings to Google's MediaPipe hand tracking library. It offers more accurate and robust hand detection compared to the OpenCV skin-color approach.

## Quick Start

### Option 1: Automated Script (Recommended)

Run the automated build script:

```bash
./scripts/build-mediapipe.sh
```

This script will:
- Install all system dependencies
- Download and build MediaPipe from source
- Set up the Rust bindings
- Configure your environment

### Option 2: Manual Build

Follow the detailed instructions in [`boid-mediapipe/README.md`](boid-mediapipe/README.md).

## System Requirements

- **OS**: Linux (Ubuntu 20.04+ recommended)
- **RAM**: At least 4GB (8GB+ recommended for building)
- **Disk Space**: ~5GB for MediaPipe source and build artifacts
- **Build Time**: 15-30 minutes on modern hardware

### Dependencies

```bash
sudo apt-get install -y \
    build-essential \
    git \
    python3 \
    python3-pip \
    libopencv-dev \
    libclang-dev \
    clang \
    cmake \
    wget

# Install Python dependencies (required for MediaPipe build)
pip3 install numpy
```

## Architecture

```
┌─────────────────────────────────────────────┐
│           boid-client (Desktop/RPi)         │
│                                             │
│  ┌─────────────────────────────────────┐   │
│  │     hand_tracker.rs                 │   │
│  │  (OpenCV OR MediaPipe interface)    │   │
│  └──────────────┬──────────────────────┘   │
│                 │                           │
│                 ▼                           │
│  ┌─────────────────────────────────────┐   │
│  │      boid-mediapipe                 │   │
│  │   (Rust FFI Wrapper)                │   │
│  └──────────────┬──────────────────────┘   │
│                 │                           │
│                 ▼                           │
│  ┌─────────────────────────────────────┐   │
│  │    wrapper.cpp (C API)              │   │
│  └──────────────┬──────────────────────┘   │
│                 │                           │
│                 ▼                           │
│  ┌─────────────────────────────────────┐   │
│  │   MediaPipe C++ Library             │   │
│  │   (Hand Tracking Pipeline)          │   │
│  └─────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

## Using MediaPipe in boid-client

Once MediaPipe is built, update `boid-client/Cargo.toml`:

```toml
[dependencies]
boid-mediapipe = { path = "../boid-mediapipe" }
# opencv = ...  # Can keep for fallback or remove
```

Update `boid-client/src/hand_tracker.rs`:

```rust
use anyhow::Result;
use boid_mediapipe::HandDetector;
use boid_shared::HandLandmarks;
use opencv::core::Mat;

pub struct HandTracker {
    detector: HandDetector,
}

impl HandTracker {
    pub fn new() -> Result<Self> {
        let detector = HandDetector::new()?;
        Ok(Self { detector })
    }

    pub fn process_frame(&mut self, frame: &Mat) -> Result<Option<HandLandmarks>> {
        let data = frame.data_bytes()?;
        let width = frame.cols();
        let height = frame.rows();
        self.detector.process_frame(data, width, height)
    }
}
```

## CI/CD Integration

The project includes a GitHub Actions workflow that builds MediaPipe in CI. See `.github/workflows/mediapipe-ci.yml`.

### Key Features

- **Caching**: MediaPipe build artifacts are cached to speed up subsequent runs
- **Fallback**: If MediaPipe fails to build, the workflow continues without it
- **Selective Building**: Only builds MediaPipe when needed

### Enabling in CI

The workflow is automatically triggered on pushes to `main` and Claude branches. No additional configuration is required.

## Performance Comparison

| Method | FPS (Desktop) | FPS (RPi 4) | Accuracy | Lighting | Skin Tone |
|--------|---------------|-------------|----------|----------|-----------|
| OpenCV | 30-60 | 15-30 | Medium | Sensitive | Very Sensitive |
| MediaPipe | 30-60 | 10-25 | High | Robust | Independent |

**Pros of MediaPipe:**
- More accurate hand landmark detection (21 points)
- Works in various lighting conditions
- Skin tone independent
- Detects hand orientation

**Cons of MediaPipe:**
- Larger binary size
- More complex build process
- Slightly lower FPS on resource-constrained devices

## Troubleshooting

### Build Fails with "Bazel not found"

```bash
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel
```

### Runtime Error: "libmediapipe_hand_tracking.so not found"

```bash
export LD_LIBRARY_PATH=/usr/local/mediapipe/lib:$LD_LIBRARY_PATH
# Or add to ~/.bashrc
```

### Bindgen Fails to Generate Bindings

```bash
export MEDIAPIPE_DIR=/opt/mediapipe
export LIBCLANG_PATH=/usr/lib/llvm-18/lib
```

### OpenCV Headers Not Found

If MediaPipe build fails with `fatal error: opencv2/core/version.hpp: No such file or directory`:

This is a common issue on Ubuntu 24.04 where OpenCV 4.x headers are installed in `/usr/include/opencv4/`. The automated build script handles this, but if building manually:

```bash
# Use Bazel flags to specify OpenCV include path
bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
    --copt=-I/usr/include/opencv4 \
    --cxxopt=-I/usr/include/opencv4 \
    mediapipe/examples/desktop/hand_tracking:hand_tracking_cpu
```

### NumPy Not Found

If MediaPipe build fails with `ModuleNotFoundError: No module named 'numpy'`:

```bash
pip3 install numpy
```

### Out of Memory During Build

Reduce Bazel's memory usage:

```bash
bazel build --local_ram_resources=2048 ...
```

## Docker Development

For a clean, reproducible build environment:

```bash
docker build -f boid-mediapipe/Dockerfile.mediapipe -t boid-rs-mediapipe .
docker run -v $(pwd):/workspace boid-rs-mediapipe cargo build --release
```

## Fallback to OpenCV

If MediaPipe build fails, the client will automatically use OpenCV-based detection if the boid-mediapipe crate is not included.

## Contributing

When adding MediaPipe-related changes:

1. Test locally with `./scripts/build-mediapipe.sh`
2. Ensure CI passes with MediaPipe workflow
3. Update documentation if adding new features
4. Test fallback behavior without MediaPipe

## References

- [MediaPipe Documentation](https://developers.google.com/mediapipe)
- [MediaPipe GitHub](https://github.com/google-ai-edge/mediapipe)
- [Bazel Build System](https://bazel.build/)
- [boid-mediapipe README](boid-mediapipe/README.md)

## License

MediaPipe is licensed under Apache 2.0. The boid-mediapipe bindings follow the same license as the boid-rs project.
