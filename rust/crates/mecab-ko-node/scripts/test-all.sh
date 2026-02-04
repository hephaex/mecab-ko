#!/bin/bash
# Comprehensive test script for mecab-ko-node

set -e

echo "Testing mecab-ko-node..."
echo ""

# Change to the crate directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR/.."

# 1. Run Rust tests
echo "=== Running Rust tests ==="
cargo test --lib
echo "✓ Rust tests passed"
echo ""

# 2. Build the native module
echo "=== Building native module (debug) ==="
npm run build:debug
echo "✓ Native module built"
echo ""

# 3. Run TypeScript/JavaScript tests
echo "=== Running Node.js tests ==="
npm test
echo "✓ Node.js tests passed"
echo ""

# 4. Run examples
echo "=== Testing CommonJS example ==="
node examples/basic.js > /dev/null
echo "✓ CommonJS example works"
echo ""

echo "=== Testing ESM example ==="
node examples/esm-example.mjs > /dev/null
echo "✓ ESM example works"
echo ""

# 5. TypeScript compilation check
echo "=== Checking TypeScript definitions ==="
npx tsc --noEmit examples/typescript-example.ts
echo "✓ TypeScript definitions are valid"
echo ""

# 6. Check code formatting
echo "=== Checking Rust formatting ==="
cargo fmt --check
echo "✓ Rust code is formatted"
echo ""

# 7. Run Clippy
echo "=== Running Clippy ==="
cargo clippy --lib -- -D warnings
echo "✓ Clippy checks passed"
echo ""

echo "All tests passed! ✓"
