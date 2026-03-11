#!/bin/bash
# Performance benchmark script for E2E tests

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
MECAB_BIN="${PROJECT_ROOT}/rust/target/release/mecab-ko"
FIXTURES_DIR="${PROJECT_ROOT}/tests/e2e/fixtures"

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Check if binary exists
if [ ! -f "$MECAB_BIN" ]; then
    log_warn "mecab-ko binary not found at $MECAB_BIN"
    log_info "Building release binary..."
    cd "${PROJECT_ROOT}/rust"
    cargo build --release --bin mecab-ko
fi

# Benchmark functions
benchmark_cli() {
    local input="$1"
    local iterations="$2"
    local description="$3"

    log_info "Benchmarking: $description"

    local start=$(date +%s%N)
    for ((i=0; i<iterations; i++)); do
        echo "$input" | "$MECAB_BIN" > /dev/null
    done
    local end=$(date +%s%N)

    local elapsed=$(( (end - start) / 1000000 ))  # Convert to ms
    local avg=$(( elapsed / iterations ))

    echo "  Total: ${elapsed}ms"
    echo "  Iterations: ${iterations}"
    echo "  Average: ${avg}ms/iter"
    echo "  Throughput: $((1000 / avg)) iter/s"
    echo ""
}

benchmark_python() {
    if ! command -v python3 &> /dev/null; then
        log_warn "Python not found, skipping Python benchmarks"
        return
    fi

    log_info "Running Python benchmarks..."

    python3 << 'EOF'
import time
import sys

try:
    import mecab_ko
except ImportError:
    print("mecab_ko not installed, skipping")
    sys.exit(0)

tagger = mecab_ko.Tagger()
text = "나는 학교에 갑니다."
iterations = 1000

start = time.time()
for _ in range(iterations):
    tagger.parse(text)
end = time.time()

elapsed_ms = (end - start) * 1000
avg_ms = elapsed_ms / iterations

print(f"  Total: {elapsed_ms:.2f}ms")
print(f"  Iterations: {iterations}")
print(f"  Average: {avg_ms:.2f}ms/iter")
print(f"  Throughput: {1000/avg_ms:.2f} iter/s")
print()
EOF
}

# Main benchmarks
echo "================================"
echo "MeCab-Ko E2E Performance Benchmarks"
echo "================================"
echo ""

log_info "CLI Benchmarks"
echo ""

# Short text
benchmark_cli "나는 학교에 갑니다." 100 "Short text (100 iterations)"

# Medium text
MEDIUM_TEXT="형태소 분석은 자연어 처리의 가장 기본적인 작업 중 하나로, 문장을 의미 있는 최소 단위인 형태소로 분리하고 각 형태소의 품사를 판별하는 과정입니다."
benchmark_cli "$MEDIUM_TEXT" 100 "Medium text (100 iterations)"

# Long text (repeat short text)
LONG_TEXT=""
for i in {1..100}; do
    LONG_TEXT="${LONG_TEXT}나는 학교에 갑니다. "
done
benchmark_cli "$LONG_TEXT" 10 "Long text (10 iterations)"

# Batch processing
log_info "Batch processing benchmark"
{
    for i in {1..1000}; do
        echo "나는 학교에 갑니다."
    done
} | time "$MECAB_BIN" > /dev/null
echo ""

# Python benchmarks
benchmark_python

# Memory usage
log_info "Memory usage test"
if command -v /usr/bin/time &> /dev/null; then
    {
        for i in {1..10000}; do
            echo "나는 학교에 갑니다."
        done
    } | /usr/bin/time -v "$MECAB_BIN" > /dev/null 2>&1 || true
fi

echo "================================"
log_info "Benchmarks complete!"
