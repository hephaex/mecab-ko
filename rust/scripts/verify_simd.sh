#!/bin/bash
# SIMD 최적화 검증 스크립트

set -e

echo "=== MeCab-Ko SIMD 최적화 검증 ==="
echo ""

# 색상 정의
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 성공/실패 카운터
PASSED=0
FAILED=0

check_step() {
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $1"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $1"
        ((FAILED++))
        return 1
    fi
}

# 1. Rust 버전 확인
echo "1. Rust 버전 확인"
rustc --version | grep -q "rustc 1." && check_step "Rust 설치됨" || check_step "Rust 설치 필요"
cargo +nightly --version > /dev/null 2>&1 && check_step "Nightly toolchain 설치됨" || check_step "Nightly toolchain 필요"
echo ""

# 2. 빌드 테스트
echo "2. 빌드 테스트"
cd "$(dirname "$0")/.."

echo -n "   SIMD feature 없이 빌드... "
cargo build --quiet --release 2>&1 | grep -q "error" && check_step "기본 빌드 실패" || check_step "기본 빌드 성공"

echo -n "   SIMD feature로 빌드... "
cargo +nightly build --quiet --release --features simd 2>&1 | grep -q "error" && check_step "SIMD 빌드 실패" || check_step "SIMD 빌드 성공"
echo ""

# 3. 테스트 실행
echo "3. 테스트 실행"

echo -n "   mecab-ko-dict SIMD 테스트... "
cargo +nightly test --quiet --package mecab-ko-dict --features simd --lib matrix::simd 2>&1 | grep -q "test result: ok" && check_step "통과" || check_step "실패"

echo -n "   mecab-ko-core SIMD 테스트... "
cargo +nightly test --quiet --package mecab-ko-core --features simd --lib viterbi::simd 2>&1 | grep -q "test result: ok" && check_step "통과" || check_step "실패"
echo ""

# 4. 코드 품질 검증
echo "4. 코드 품질 검증"

echo -n "   Clippy (SIMD)... "
cargo +nightly clippy --quiet --features simd -- -D warnings 2>&1 | grep -q "error" && check_step "경고 있음" || check_step "경고 없음"

echo -n "   Formatting 검사... "
cargo fmt -- --check 2>&1 | grep -q "Diff" && check_step "포맷 필요" || check_step "포맷 정상"
echo ""

# 5. 문서 검증
echo "5. 문서 검증"

FILES=(
    "docs/SIMD_OPTIMIZATION.md"
    "docs/PHASE6_SIMD_SUMMARY.md"
    "docs/phase6/README.md"
    "crates/mecab-ko-dict/src/matrix/simd.rs"
    "crates/mecab-ko-core/src/viterbi/simd.rs"
)

for file in "${FILES[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC} $file"
        ((PASSED++))
    else
        echo -e "${RED}✗${NC} $file (없음)"
        ((FAILED++))
    fi
done
echo ""

# 6. 예제 실행
echo "6. 예제 실행"
echo -n "   simd_demo 예제... "
timeout 10 cargo +nightly run --quiet --example simd_demo --features simd --release 2>&1 | grep -q "SIMD 최적화 활성화됨" && check_step "정상 실행" || check_step "실행 실패"
echo ""

# 결과 요약
echo "==================================="
echo -e "검증 완료: ${GREEN}${PASSED}${NC} 통과, ${RED}${FAILED}${NC} 실패"

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ 모든 검증 통과!${NC}"
    exit 0
else
    echo -e "${RED}✗ 일부 검증 실패${NC}"
    exit 1
fi
