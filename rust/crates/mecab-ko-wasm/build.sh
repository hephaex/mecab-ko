#!/bin/bash
# MeCab-Ko WASM Build Script
set -e

echo "========================================"
echo "Building MeCab-Ko WASM..."
echo "========================================"

# Check for wasm-pack
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Build for bundler (webpack, rollup, etc.)
echo ""
echo "[1/3] Building for bundler (pkg/)..."
wasm-pack build --target bundler --out-dir pkg --release

# Build for Node.js
echo ""
echo "[2/3] Building for Node.js (pkg-node/)..."
wasm-pack build --target nodejs --out-dir pkg-node --release

# Build for web (ES modules)
echo ""
echo "[3/3] Building for web (pkg-web/)..."
wasm-pack build --target web --out-dir pkg-web --release

# Copy TypeScript definitions
echo ""
echo "Copying TypeScript definitions..."
cp index.d.ts pkg/ 2>/dev/null || true
cp index.d.ts pkg-node/ 2>/dev/null || true
cp index.d.ts pkg-web/ 2>/dev/null || true

echo ""
echo "========================================"
echo "Build complete!"
echo "========================================"
echo ""
echo "Output directories:"
echo "  - Bundler (webpack/rollup): ./pkg/"
echo "  - Node.js:                  ./pkg-node/"
echo "  - Web (ES modules):         ./pkg-web/"
echo ""
echo "Usage:"
echo "  npm publish ./pkg          # Publish to npm"
echo "  node examples/node.js      # Run Node.js example"
echo ""
