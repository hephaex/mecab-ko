#!/usr/bin/env bash
# Flamegraph 생성 스크립트
# cargo-flamegraph 필요: cargo install flamegraph

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

# 사용법 출력
usage() {
    echo "Usage: $0 <benchmark_name>"
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
    echo ""
    echo "Note: cargo-flamegraph must be installed"
    echo "  cargo install flamegraph"
    exit 1
}

if [ $# -lt 1 ]; then
    usage
fi

BENCH_NAME="$1"

# cargo-flamegraph 확인
if ! command -v cargo-flamegraph &> /dev/null; then
    echo "Error: cargo-flamegraph not found"
    echo "Please install: cargo install flamegraph"
    exit 1
fi

cd "$PROJECT_DIR"

echo "=========================================="
echo "Flamegraph 생성 중: $BENCH_NAME"
echo "=========================================="
echo ""

# Flamegraph 생성
cargo flamegraph --bench "$BENCH_NAME" -o "flamegraph-${BENCH_NAME}.svg"

OUTPUT_FILE="$PROJECT_DIR/flamegraph-${BENCH_NAME}.svg"

echo ""
echo "=========================================="
echo "Flamegraph 생성 완료!"
echo "=========================================="
echo ""
echo "출력 파일: $OUTPUT_FILE"
echo ""
