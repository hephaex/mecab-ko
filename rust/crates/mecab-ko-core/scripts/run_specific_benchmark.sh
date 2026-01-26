#!/usr/bin/env bash
# 특정 벤치마크만 실행하는 스크립트

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 사용법 출력
usage() {
    echo "Usage: $0 <benchmark_name> [benchmark_filter]"
    echo ""
    echo "Available benchmarks:"
    echo "  - tokenizer_bench"
    echo "  - lattice_bench"
    echo "  - viterbi_bench"
    echo "  - memory_bench"
    echo "  - comparison_bench"
    echo ""
    echo "Examples:"
    echo "  $0 tokenizer_bench"
    echo "  $0 tokenizer_bench tokenize_basic"
    echo "  $0 lattice_bench node_addition"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

BENCH_NAME="$1"
FILTER="${2:-}"

cd "$PROJECT_DIR"

echo "=========================================="
echo "벤치마크 실행: $BENCH_NAME"
if [ -n "$FILTER" ]; then
    echo "필터: $FILTER"
fi
echo "=========================================="
echo ""

if [ -n "$FILTER" ]; then
    cargo bench --bench "$BENCH_NAME" -- "$FILTER"
else
    cargo bench --bench "$BENCH_NAME"
fi

echo ""
echo "=========================================="
echo "벤치마크 완료!"
echo "=========================================="
