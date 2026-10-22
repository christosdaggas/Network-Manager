#!/bin/bash
# Build script for Network Manager AppImage
# Requires: appimage-builder, cargo

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Building Network Manager AppImage ==="
echo "Project root: $PROJECT_ROOT"

# Ensure we're in the appimage directory
cd "$SCRIPT_DIR"

# Build the release binary
echo "Building release binary..."
cd "$PROJECT_ROOT"
cargo build --release -p network-manager

# Return to appimage directory
cd "$SCRIPT_DIR"

# Check if appimage-builder is installed
if ! command -v appimage-builder &> /dev/null; then
    echo "Error: appimage-builder is not installed"
    echo "Install it with: pip install appimage-builder"
    exit 1
fi

# Build the AppImage
echo "Building AppImage..."
appimage-builder --recipe AppImageBuilder.yml

echo "=== AppImage build complete ==="
ls -la *.AppImage 2>/dev/null || echo "AppImage file not found in current directory"
