#!/bin/bash
set -e

echo "=== Testing nori_compat module ==="
cd "$(dirname "$0")"

echo ""
echo "1. Running clippy..."
cargo clippy -p mecab-ko-core --lib -- -D warnings

echo ""
echo "2. Running tests..."
cargo test -p mecab-ko-core nori_compat --lib

echo ""
echo "3. Running example (if dictionary available)..."
if cargo run --example compound_noun_demo 2>&1 | grep -q "Failed to create tokenizer"; then
    echo "   Note: Dictionary not available, skipping example"
else
    cargo run --example compound_noun_demo | head -100
fi

echo ""
echo "=== All checks passed ==="
