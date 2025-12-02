# boid-mediapipe

Rust bindings for Google MediaPipe hand tracking, specifically designed for the boid-rs project.

## Overview

This crate provides Rust FFI bindings to MediaPipe's hand tracking functionality. It wraps the C++ MediaPipe library with a C API layer, which is then exposed to Rust through bindgen.

## Prerequisites

### System Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install -y \
    build-essential \
    git \
    python3 \
    python3-pip \
    libopencv-dev \
    cmake \
    wget

# Install Python dependencies (required for MediaPipe build)
pip3 install numpy

# Install Bazelisk (Bazel version manager)
wget https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel
```

## Building MediaPipe from Source

### 1. Clone MediaPipe Repository

```bash
cd /opt
sudo git clone https://github.com/google-ai-edge/mediapipe.git
cd mediapipe
```

### 2. Build Hand Tracking Example

```bash
# Build the hand tracking library
bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
    mediapipe/examples/desktop/hand_tracking:hand_tracking_cpu

# This will create the binary at:
# bazel-bin/mediapipe/examples/desktop/hand_tracking/hand_tracking_cpu
```

### 3. Build as Shared Library

To use with Rust, we need to build MediaPipe as a shared library. Create a custom BUILD file:

```bash
# Create mediapipe/examples/desktop/hand_tracking/BUILD.rust
cat > mediapipe/examples/desktop/hand_tracking/BUILD.rust << 'EOF'
cc_binary(
    name = "libmediapipe_hand_tracking.so",
    linkshared = True,
    deps = [
        "//mediapipe/framework:calculator_framework",
        "//mediapipe/framework/formats:image_frame",
        "//mediapipe/framework/formats:image_frame_opencv",
        "//mediapipe/framework/port:opencv_core",
        "//mediapipe/framework/port:opencv_imgproc",
        "//mediapipe/calculators/core:flow_limiter_calculator",
        "//mediapipe/calculators/util:landmarks_to_render_data_calculator",
        "//mediapipe/framework/formats:landmark_cc_proto",
    ],
)
EOF

# Build the shared library
bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
    mediapipe/examples/desktop/hand_tracking:libmediapipe_hand_tracking.so
```

### 4. Install MediaPipe Libraries

```bash
# Create installation directory
sudo mkdir -p /usr/local/mediapipe/lib
sudo mkdir -p /usr/local/mediapipe/include

# Copy the built library
sudo cp bazel-bin/mediapipe/examples/desktop/hand_tracking/libmediapipe_hand_tracking.so \
    /usr/local/mediapipe/lib/

# Copy headers (recursive)
sudo cp -r mediapipe /usr/local/mediapipe/include/

# Set library path
echo "/usr/local/mediapipe/lib" | sudo tee /etc/ld.so.conf.d/mediapipe.conf
sudo ldconfig
```

## Building boid-mediapipe

### Environment Variables

Set the MediaPipe installation path:

```bash
export MEDIAPIPE_DIR=/opt/mediapipe
export LD_LIBRARY_PATH=/usr/local/mediapipe/lib:$LD_LIBRARY_PATH
```

### Build the Rust Crate

```bash
cd boid-mediapipe
cargo build --release
```

## Using in boid-client

### Update boid-client/Cargo.toml

```toml
[dependencies]
boid-mediapipe = { path = "../boid-mediapipe" }
```

### Update hand_tracker.rs

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
        // Convert Mat to raw bytes
        let data = frame.data_bytes()?;
        let width = frame.cols();
        let height = frame.rows();

        self.detector.process_frame(data, width, height)
    }
}
```

## CI/CD Integration

### GitHub Actions Workflow

Add to `.github/workflows/ci.yml`:

```yaml
name: CI

on: [push, pull_request]

jobs:
  build-with-mediapipe:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install system dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y build-essential libopencv-dev cmake

      - name: Install Bazelisk
        run: |
          wget https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64
          chmod +x bazelisk-linux-amd64
          sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel

      - name: Cache MediaPipe
        id: cache-mediapipe
        uses: actions/cache@v3
        with:
          path: /opt/mediapipe
          key: mediapipe-${{ runner.os }}-${{ hashFiles('.github/mediapipe-version.txt') }}

      - name: Build MediaPipe
        if: steps.cache-mediapipe.outputs.cache-hit != 'true'
        run: |
          cd /opt
          sudo git clone https://github.com/google-ai-edge/mediapipe.git
          cd mediapipe
          bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
            mediapipe/examples/desktop/hand_tracking:hand_tracking_cpu

      - name: Install MediaPipe
        run: |
          sudo mkdir -p /usr/local/mediapipe/lib
          sudo cp /opt/mediapipe/bazel-bin/mediapipe/examples/desktop/hand_tracking/libmediapipe_hand_tracking.so \
            /usr/local/mediapipe/lib/ || true
          echo "/usr/local/mediapipe/lib" | sudo tee /etc/ld.so.conf.d/mediapipe.conf
          sudo ldconfig

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build project
        env:
          MEDIAPIPE_DIR: /opt/mediapipe
        run: cargo build --release

      - name: Run tests
        env:
          MEDIAPIPE_DIR: /opt/mediapipe
        run: cargo test --workspace
```

### Docker Support

Create `Dockerfile.mediapipe`:

```dockerfile
FROM ubuntu:24.04

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    python3 \
    python3-pip \
    libopencv-dev \
    cmake \
    wget \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Bazelisk
RUN wget https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64 && \
    chmod +x bazelisk-linux-amd64 && \
    mv bazelisk-linux-amd64 /usr/local/bin/bazel

# Clone and build MediaPipe
WORKDIR /opt
RUN git clone https://github.com/google-ai-edge/mediapipe.git
WORKDIR /opt/mediapipe
RUN bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
    mediapipe/examples/desktop/hand_tracking:hand_tracking_cpu

# Install MediaPipe libraries
RUN mkdir -p /usr/local/mediapipe/lib && \
    cp bazel-bin/mediapipe/examples/desktop/hand_tracking/libmediapipe_hand_tracking.so \
    /usr/local/mediapipe/lib/ || true && \
    echo "/usr/local/mediapipe/lib" > /etc/ld.so.conf.d/mediapipe.conf && \
    ldconfig

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set environment variables
ENV MEDIAPIPE_DIR=/opt/mediapipe
ENV LD_LIBRARY_PATH=/usr/local/mediapipe/lib:$LD_LIBRARY_PATH

WORKDIR /workspace
```

Build and use:

```bash
docker build -f Dockerfile.mediapipe -t boid-rs-mediapipe .
docker run -v $(pwd):/workspace boid-rs-mediapipe cargo build --release
```

## Troubleshooting

### Library Not Found

If you get errors about missing `libmediapipe_hand_tracking.so`:

```bash
export LD_LIBRARY_PATH=/usr/local/mediapipe/lib:$LD_LIBRARY_PATH
# Or add permanently to ~/.bashrc
```

### Build Errors

If bindgen fails to find headers:

```bash
export MEDIAPIPE_DIR=/opt/mediapipe
export CPLUS_INCLUDE_PATH=/opt/mediapipe:$CPLUS_INCLUDE_PATH
```

### Bazel Issues

Clear Bazel cache if you encounter strange build errors:

```bash
cd /opt/mediapipe
bazel clean --expunge
```

## Performance Notes

- MediaPipe hand tracking typically runs at 30-60 FPS on modern CPUs
- GPU acceleration is not used (MEDIAPIPE_DISABLE_GPU=1)
- For Raspberry Pi, consider reducing frame resolution for better performance

## License

This crate is part of boid-rs and follows the same license. MediaPipe itself is licensed under Apache 2.0.

## References

- [MediaPipe GitHub](https://github.com/google-ai-edge/mediapipe)
- [MediaPipe Hand Tracking](https://developers.google.com/mediapipe/solutions/vision/hand_landmarker)
- [Bazel Build System](https://bazel.build/)
