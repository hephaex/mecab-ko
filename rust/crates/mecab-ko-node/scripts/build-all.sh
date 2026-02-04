#!/bin/bash
# Build script for all platforms
# This script is used for CI/CD to build native modules for all supported platforms

set -e

echo "Building mecab-ko-node for all platforms..."
echo ""

# Check if napi CLI is installed
if ! command -v napi &> /dev/null; then
    echo "Error: @napi-rs/cli is not installed"
    echo "Run: npm install -g @napi-rs/cli"
    exit 1
fi

# Platform targets
TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

# Build for each target
for target in "${TARGETS[@]}"; do
    echo "Building for ${target}..."

    # Add target to rustup if not present
    rustup target add "${target}" 2>/dev/null || true

    # Build with napi
    if napi build --platform --release --target "${target}" --cargo-name mecab-ko-node; then
        echo "✓ Built successfully for ${target}"
    else
        echo "✗ Failed to build for ${target}"
        # Don't fail the entire build if one target fails
        # Some targets may not be available on current host
    fi

    echo ""
done

echo "Build complete!"
echo ""
echo "Artifacts:"
ls -lh *.node 2>/dev/null || echo "No .node files found in current directory"
