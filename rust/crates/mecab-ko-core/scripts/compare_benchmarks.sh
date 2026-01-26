#!/usr/bin/env bash
# 벤치마크 결과 비교 스크립트

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 사용법 출력
usage() {
    echo "Usage: $0 <baseline_name> <benchmark_name>"
    echo ""
    echo "Compare benchmark results between baseline and current"
    echo ""
    echo "Examples:"
    echo "  $0 baseline tokenizer_bench"
    echo ""
    echo "This will:"
    echo "  1. Save current results as baseline"
    echo "  2. Make changes to your code"
    echo "  3. Run this script to compare"
    exit 1
}

if [ $# -lt 2 ]; then
    usage
fi

BASELINE="$1"
BENCH_NAME="$2"

cd "$PROJECT_DIR"

RESULTS_DIR="$PROJECT_DIR/target/criterion"

echo "=========================================="
echo "벤치마크 비교"
echo "=========================================="
echo ""
echo "Baseline: $BASELINE"
echo "Benchmark: $BENCH_NAME"
echo ""

# baseline 저장
if [ "$BASELINE" = "save" ]; then
    echo "현재 결과를 baseline으로 저장 중..."
    cargo bench --bench "$BENCH_NAME" -- --save-baseline "$BENCH_NAME-baseline"
    echo "Baseline 저장 완료: $BENCH_NAME-baseline"
    exit 0
fi

# 비교 실행
echo "비교 실행 중..."
cargo bench --bench "$BENCH_NAME" -- --baseline "$BASELINE"

echo ""
echo "=========================================="
echo "비교 완료!"
echo "=========================================="
echo ""
echo "결과 위치: $RESULTS_DIR"
