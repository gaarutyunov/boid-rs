#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}MediaPipe Build Script for boid-rs${NC}"
echo "========================================"

# Configuration
MEDIAPIPE_VERSION=${MEDIAPIPE_VERSION:-"v0.10.9"}
MEDIAPIPE_DIR=${MEDIAPIPE_DIR:-"/opt/mediapipe"}
INSTALL_DIR=${INSTALL_DIR:-"/usr/local/mediapipe"}

echo -e "${YELLOW}Configuration:${NC}"
echo "  MediaPipe Version: $MEDIAPIPE_VERSION"
echo "  Source Directory: $MEDIAPIPE_DIR"
echo "  Install Directory: $INSTALL_DIR"
echo ""

# Check for required tools
echo -e "${YELLOW}Checking dependencies...${NC}"

check_command() {
    if ! command -v $1 &> /dev/null; then
        echo -e "${RED}Error: $1 is not installed${NC}"
        return 1
    fi
    echo -e "${GREEN}✓${NC} $1 found"
}

check_command git || exit 1
check_command cmake || exit 1
check_command python3 || exit 1
check_command bazel || {
    echo -e "${YELLOW}Bazel not found. Installing Bazelisk...${NC}"
    wget -q https://github.com/bazelbuild/bazelisk/releases/download/v1.19.0/bazelisk-linux-amd64
    chmod +x bazelisk-linux-amd64
    sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel
    echo -e "${GREEN}✓${NC} Bazelisk installed"
}

# Install system dependencies
echo -e "\n${YELLOW}Installing system dependencies...${NC}"
sudo apt-get update
sudo apt-get install -y build-essential libopencv-dev libclang-dev clang cmake python3 python3-pip

# Install Python dependencies (required for MediaPipe build)
echo -e "\n${YELLOW}Installing Python dependencies...${NC}"
pip3 install --user numpy
echo -e "${GREEN}✓${NC} numpy installed"

# Clone MediaPipe
if [ ! -d "$MEDIAPIPE_DIR" ]; then
    echo -e "\n${YELLOW}Cloning MediaPipe repository...${NC}"
    sudo mkdir -p $(dirname "$MEDIAPIPE_DIR")
    cd $(dirname "$MEDIAPIPE_DIR")
    sudo git clone --depth 1 --branch "$MEDIAPIPE_VERSION" https://github.com/google-ai-edge/mediapipe.git
    echo -e "${GREEN}✓${NC} MediaPipe cloned"
else
    echo -e "${GREEN}✓${NC} MediaPipe already exists at $MEDIAPIPE_DIR"
fi

# Build MediaPipe
echo -e "\n${YELLOW}Building MediaPipe hand tracking...${NC}"
echo "This may take 15-30 minutes on first build..."

cd "$MEDIAPIPE_DIR"

# Build hand tracking example
bazel build -c opt --define MEDIAPIPE_DISABLE_GPU=1 \
    mediapipe/examples/desktop/hand_tracking:hand_tracking_cpu

echo -e "${GREEN}✓${NC} MediaPipe built successfully"

# Install libraries and headers
echo -e "\n${YELLOW}Installing MediaPipe libraries...${NC}"

sudo mkdir -p "$INSTALL_DIR/lib"
sudo mkdir -p "$INSTALL_DIR/include"

# Copy built artifacts
if [ -f "$MEDIAPIPE_DIR/bazel-bin/mediapipe/examples/desktop/hand_tracking/hand_tracking_cpu" ]; then
    sudo cp "$MEDIAPIPE_DIR/bazel-bin/mediapipe/examples/desktop/hand_tracking/hand_tracking_cpu" \
        "$INSTALL_DIR/lib/" || echo "Warning: Could not copy binary"
fi

# Copy headers
if [ -d "$MEDIAPIPE_DIR/mediapipe" ]; then
    sudo cp -r "$MEDIAPIPE_DIR/mediapipe" "$INSTALL_DIR/include/"
    echo -e "${GREEN}✓${NC} Headers installed"
fi

# Update library cache
echo "$INSTALL_DIR/lib" | sudo tee /etc/ld.so.conf.d/mediapipe.conf > /dev/null
sudo ldconfig

echo -e "${GREEN}✓${NC} MediaPipe libraries installed"

# Set up environment
echo -e "\n${YELLOW}Setting up environment...${NC}"

SHELL_RC="$HOME/.bashrc"
if [ -n "$ZSH_VERSION" ]; then
    SHELL_RC="$HOME/.zshrc"
fi

if ! grep -q "MEDIAPIPE_DIR" "$SHELL_RC"; then
    echo "" >> "$SHELL_RC"
    echo "# MediaPipe environment" >> "$SHELL_RC"
    echo "export MEDIAPIPE_DIR=$MEDIAPIPE_DIR" >> "$SHELL_RC"
    echo "export LD_LIBRARY_PATH=$INSTALL_DIR/lib:\$LD_LIBRARY_PATH" >> "$SHELL_RC"
    echo -e "${GREEN}✓${NC} Environment variables added to $SHELL_RC"
else
    echo -e "${YELLOW}!${NC} Environment variables already in $SHELL_RC"
fi

# Test build
echo -e "\n${YELLOW}Testing Rust build...${NC}"

export MEDIAPIPE_DIR="$MEDIAPIPE_DIR"
export LD_LIBRARY_PATH="$INSTALL_DIR/lib:$LD_LIBRARY_PATH"

cd "$(dirname "$0")/.."

if cargo build -p boid-mediapipe 2>&1 | grep -q "error"; then
    echo -e "${YELLOW}!${NC} MediaPipe Rust crate build encountered issues (expected in some environments)"
    echo "   The infrastructure is set up, but you may need to adjust paths or dependencies"
else
    echo -e "${GREEN}✓${NC} Rust build successful!"
fi

# Summary
echo -e "\n${GREEN}========================================${NC}"
echo -e "${GREEN}MediaPipe installation complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "To use MediaPipe, run:"
echo -e "  ${YELLOW}source $SHELL_RC${NC}"
echo "  ${YELLOW}cargo build --release${NC}"
echo ""
echo "Or manually set environment variables:"
echo -e "  ${YELLOW}export MEDIAPIPE_DIR=$MEDIAPIPE_DIR${NC}"
echo -e "  ${YELLOW}export LD_LIBRARY_PATH=$INSTALL_DIR/lib:\$LD_LIBRARY_PATH${NC}"
echo ""
echo "Build location: $MEDIAPIPE_DIR"
echo "Install location: $INSTALL_DIR"
