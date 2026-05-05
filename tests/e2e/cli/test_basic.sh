#!/bin/bash
set -euo pipefail

echo "=== MeCab-Ko CLI E2E Tests ==="

MECAB_BIN="${MECAB_BIN:-cargo run --manifest-path rust/Cargo.toml --release --bin mecab --}"

echo "Test 1: Basic tokenization"
echo "안녕하세요" | $MECAB_BIN && echo "PASS" || echo "FAIL"

echo "Test 2: Version flag"
$MECAB_BIN --version && echo "PASS" || echo "FAIL"

echo "=== CLI E2E Tests Complete ==="
