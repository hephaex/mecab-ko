#!/usr/bin/env bash
#
# MeCab-Ko 종합 벤치마크 실행 스크립트
#
# 사용법:
#   ./run_benchmarks.sh [options]
#
# 옵션:
#   --all              모든 벤치마크 실행 (기본)
#   --quick            빠른 벤치마크만 실행
#   --save-baseline    베이스라인 저장
#   --compare          이전 베이스라인과 비교
#   --output-json      JSON 형식으로 결과 출력
#   --output-csv       CSV 형식으로 결과 출력
#   --help             도움말 출력

set -euo pipefail

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 기본 설정
RUN_MODE="all"
SAVE_BASELINE=""
COMPARE_BASELINE=""
OUTPUT_JSON=false
OUTPUT_CSV=false
SAMPLE_SIZE=""

# 함수: 사용법 출력
usage() {
    cat << EOF
MeCab-Ko 벤치마크 실행 스크립트

사용법: $0 [옵션]

옵션:
    --all              모든 벤치마크 실행 (기본)
    --quick            빠른 벤치마크만 실행 (샘플 크기 축소)
    --core             핵심 벤치마크만 실행
    --save-baseline NAME   베이스라인을 NAME으로 저장
    --compare NAME     NAME 베이스라인과 비교
    --output-json      JSON 형식으로 결과 출력
    --output-csv       CSV 형식으로 결과 출력
    --sample-size N    샘플 크기 설정
    --help             이 도움말 출력

예제:
    $0                           # 모든 벤치마크 실행
    $0 --quick                   # 빠른 테스트
    $0 --save-baseline main      # 베이스라인 저장
    $0 --compare main            # 베이스라인과 비교
    $0 --core --output-json      # 핵심 벤치마크, JSON 출력

EOF
}

# 함수: 로그 출력
log() {
    echo -e "${BLUE}[$(date +'%Y-%m-%d %H:%M:%S')]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*"
}

# 인자 파싱
while [[ $# -gt 0 ]]; do
    case $1 in
        --all)
            RUN_MODE="all"
            shift
            ;;
        --quick)
            RUN_MODE="quick"
            SAMPLE_SIZE="10"
            shift
            ;;
        --core)
            RUN_MODE="core"
            shift
            ;;
        --save-baseline)
            SAVE_BASELINE="$2"
            shift 2
            ;;
        --compare)
            COMPARE_BASELINE="$2"
            shift 2
            ;;
        --output-json)
            OUTPUT_JSON=true
            shift
            ;;
        --output-csv)
            OUTPUT_CSV=true
            shift
            ;;
        --sample-size)
            SAMPLE_SIZE="$2"
            shift 2
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "알 수 없는 옵션: $1"
            usage
            exit 1
            ;;
    esac
done

# 벤치마크 목록 정의
ALL_BENCHMARKS=(
    "cold_start_bench"
    "batch_bench"
    "memory_bench"
    "normalization_bench"
    "comparison_bench"
    "tokenizer_bench"
    "trie_bench"
    "matrix_bench"
    "viterbi_bench"
)

CORE_BENCHMARKS=(
    "tokenizer_bench"
    "trie_bench"
    "matrix_bench"
    "viterbi_bench"
)

QUICK_BENCHMARKS=(
    "tokenizer_bench"
    "comparison_bench"
)

# 실행할 벤치마크 선택
case $RUN_MODE in
    all)
        BENCHMARKS=("${ALL_BENCHMARKS[@]}")
        ;;
    core)
        BENCHMARKS=("${CORE_BENCHMARKS[@]}")
        ;;
    quick)
        BENCHMARKS=("${QUICK_BENCHMARKS[@]}")
        ;;
esac

# 시작 메시지
log "MeCab-Ko 벤치마크 시작"
log "모드: $RUN_MODE"
log "벤치마크 수: ${#BENCHMARKS[@]}"

# Criterion 옵션 구성
CRITERION_OPTS=""
if [[ -n "$SAVE_BASELINE" ]]; then
    CRITERION_OPTS="--save-baseline $SAVE_BASELINE"
    log "베이스라인 저장: $SAVE_BASELINE"
fi

if [[ -n "$COMPARE_BASELINE" ]]; then
    CRITERION_OPTS="--baseline $COMPARE_BASELINE"
    log "베이스라인 비교: $COMPARE_BASELINE"
fi

if [[ -n "$SAMPLE_SIZE" ]]; then
    CRITERION_OPTS="$CRITERION_OPTS --sample-size $SAMPLE_SIZE"
    log "샘플 크기: $SAMPLE_SIZE"
fi

# 결과 디렉토리 생성
RESULTS_DIR="benchmark_results_$(date +'%Y%m%d_%H%M%S')"
mkdir -p "$RESULTS_DIR"
log "결과 저장 디렉토리: $RESULTS_DIR"

# 시스템 정보 수집
log "시스템 정보 수집 중..."
{
    echo "=== System Information ==="
    echo "Date: $(date)"
    echo "Hostname: $(hostname)"
    echo "OS: $(uname -s) $(uname -r)"
    echo "CPU: $(lscpu | grep 'Model name' || echo 'N/A')"
    echo "Memory: $(free -h | grep 'Mem:' || echo 'N/A')"
    echo "Rust: $(rustc --version)"
    echo "Cargo: $(cargo --version)"
    echo ""
} > "$RESULTS_DIR/system_info.txt"

# 벤치마크 실행
log "벤치마크 실행 중..."
FAILED_BENCHMARKS=()

for bench in "${BENCHMARKS[@]}"; do
    log "실행 중: $bench"

    if cargo bench --bench "$bench" -- $CRITERION_OPTS 2>&1 | tee "$RESULTS_DIR/${bench}.log"; then
        log_success "$bench 완료"
    else
        log_error "$bench 실패"
        FAILED_BENCHMARKS+=("$bench")
    fi

    echo ""
done

# 결과 요약
log "벤치마크 실행 완료"
log "성공: $((${#BENCHMARKS[@]} - ${#FAILED_BENCHMARKS[@]}))/${#BENCHMARKS[@]}"

if [[ ${#FAILED_BENCHMARKS[@]} -gt 0 ]]; then
    log_warning "실패한 벤치마크: ${FAILED_BENCHMARKS[*]}"
fi

# Criterion HTML 보고서 복사
if [[ -d "target/criterion" ]]; then
    log "Criterion 보고서 복사 중..."
    cp -r target/criterion "$RESULTS_DIR/"
    log_success "보고서 저장: $RESULTS_DIR/criterion/report/index.html"
fi

# JSON 출력
if [[ "$OUTPUT_JSON" == true ]]; then
    log "JSON 형식으로 결과 내보내는 중..."
    # Criterion은 자동으로 JSON 생성
    if [[ -d "target/criterion" ]]; then
        find target/criterion -name "benchmark.json" -exec cp {} "$RESULTS_DIR/" \;
        log_success "JSON 저장: $RESULTS_DIR/benchmark.json"
    fi
fi

# CSV 출력 (간단한 요약)
if [[ "$OUTPUT_CSV" == true ]]; then
    log "CSV 형식으로 요약 생성 중..."
    CSV_FILE="$RESULTS_DIR/summary.csv"

    {
        echo "Benchmark,Status"
        for bench in "${BENCHMARKS[@]}"; do
            if [[ " ${FAILED_BENCHMARKS[*]} " =~ " ${bench} " ]]; then
                echo "$bench,FAILED"
            else
                echo "$bench,SUCCESS"
            fi
        done
    } > "$CSV_FILE"

    log_success "CSV 저장: $CSV_FILE"
fi

# 최종 요약
echo ""
echo "========================================="
echo "벤치마크 완료!"
echo "========================================="
echo "결과 디렉토리: $RESULTS_DIR"
echo ""
echo "HTML 보고서 보기:"
echo "  firefox $RESULTS_DIR/criterion/report/index.html"
echo ""

if [[ ${#FAILED_BENCHMARKS[@]} -eq 0 ]]; then
    log_success "모든 벤치마크 성공"
    exit 0
else
    log_error "${#FAILED_BENCHMARKS[@]}개 벤치마크 실패"
    exit 1
fi
