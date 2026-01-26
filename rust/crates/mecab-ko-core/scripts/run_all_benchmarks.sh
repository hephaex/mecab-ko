#!/usr/bin/env bash
# 모든 벤치마크 실행 스크립트

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=========================================="
echo "MeCab-Ko Core 벤치마크 실행"
echo "=========================================="
echo ""

cd "$PROJECT_DIR"

# 벤치마크 목록
BENCHMARKS=(
    "tokenizer_bench"
    "lattice_bench"
    "viterbi_bench"
    "memory_bench"
    "comparison_bench"
)

# 결과 디렉토리 생성
RESULTS_DIR="$PROJECT_DIR/target/criterion"
mkdir -p "$RESULTS_DIR"

echo "빌드 중..."
cargo build --release --benches
echo "빌드 완료!"
echo ""

# 각 벤치마크 실행
for bench in "${BENCHMARKS[@]}"; do
    echo "=========================================="
    echo "실행 중: $bench"
    echo "=========================================="
    cargo bench --bench "$bench" -- --noplot
    echo ""
done

echo "=========================================="
echo "모든 벤치마크 완료!"
echo "=========================================="
echo ""
echo "결과 위치: $RESULTS_DIR"
echo "HTML 리포트 보기: file://$RESULTS_DIR/report/index.html"
echo ""
