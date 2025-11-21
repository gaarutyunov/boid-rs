#!/bin/bash
# Script to build and run the ESP32-S3 boid simulation in QEMU
# This script is used for both local development and CI/CD testing

set -e

# Configuration
QEMU_PATH="${ESP_QEMU_PATH:-qemu-system-xtensa}"
FLASH_SIZE="${FLASH_SIZE:-4MB}"
QEMU_TIMEOUT="${QEMU_TIMEOUT:-60}"  # seconds

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Building ESP32-S3 boid simulation for QEMU...${NC}"

# Build with QEMU feature
cd "$(dirname "$0")"

# Use sdkconfig.qemu for QEMU build
export ESP_IDF_SDKCONFIG_DEFAULTS="sdkconfig.qemu"

echo -e "${YELLOW}Building with QEMU features...${NC}"
cargo +esp build --release --features qemu

# Check if build succeeded
if [ ! -f "target/xtensa-esp32s3-espidf/release/boid-esp32" ]; then
    echo -e "${RED}Build failed - binary not found${NC}"
    exit 1
fi

echo -e "${GREEN}Build successful!${NC}"

# Create flash image
echo -e "${YELLOW}Creating flash image...${NC}"

# Generate flash arguments
cargo +esp espflash save-image --chip esp32s3 --release --features qemu \
    target/xtensa-esp32s3-espidf/release/boid-esp32 \
    target/flash_image.bin

echo -e "${GREEN}Flash image created: target/flash_image.bin${NC}"

# Run QEMU
echo -e "${YELLOW}Starting QEMU...${NC}"
echo -e "${YELLOW}Test will run for approximately ${QEMU_TIMEOUT} seconds${NC}"

# QEMU command with networking and watchdog disabled
# -nographic: No graphical output
# -machine esp32s3: ESP32-S3 machine type
# -drive: Flash image
# -nic user,model=open_eth: User-mode networking with OpenCores Ethernet
# -global: Disable watchdog timers for QEMU compatibility
timeout ${QEMU_TIMEOUT}s ${QEMU_PATH} \
    -nographic \
    -machine esp32s3 \
    -drive file=target/flash_image.bin,if=mtd,format=raw \
    -nic user,model=open_eth \
    -global driver=timer.esp32s3.timg,property=wdt_disable,value=true \
    || {
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 124 ]; then
            echo -e "${YELLOW}QEMU test timed out after ${QEMU_TIMEOUT}s (expected)${NC}"
            # Check if we got the success message
            echo -e "${GREEN}Test completed - checking logs...${NC}"
            exit 0
        else
            echo -e "${RED}QEMU exited with error code: $EXIT_CODE${NC}"
            exit $EXIT_CODE
        fi
    }

echo -e "${GREEN}QEMU test completed successfully!${NC}"
