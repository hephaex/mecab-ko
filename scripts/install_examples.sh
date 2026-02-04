#!/usr/bin/env bash
#
# MeCab-Ko 실전 예제 설치 스크립트
#
# 사용법:
#   ./scripts/install_examples.sh

set -euo pipefail

# 색상 정의
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 프로젝트 루트 디렉토리 확인
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "MeCab-Ko 실전 예제 설치 스크립트"
echo "================================"
echo ""

# 소스 및 대상 디렉토리
EXAMPLES_SOURCE="${PROJECT_ROOT}/examples"
EXAMPLES_TARGET="${PROJECT_ROOT}/rust/crates/mecab-ko-core/examples"

# 디렉토리 존재 확인
if [ ! -d "${EXAMPLES_SOURCE}" ]; then
    echo -e "${RED}오류: 소스 디렉토리를 찾을 수 없습니다: ${EXAMPLES_SOURCE}${NC}"
    exit 1
fi

if [ ! -d "${EXAMPLES_TARGET}" ]; then
    echo -e "${RED}오류: 대상 디렉토리를 찾을 수 없습니다: ${EXAMPLES_TARGET}${NC}"
    exit 1
fi

# 복사할 파일 목록
EXAMPLE_FILES=(
    "text_preprocessing.rs"
    "keyword_extraction.rs"
    "search_tokenizer.rs"
)

echo "다음 예제 파일들을 복사합니다:"
for file in "${EXAMPLE_FILES[@]}"; do
    echo "  - ${file}"
done
echo ""

# 복사 실행
COPIED_COUNT=0
SKIPPED_COUNT=0

for file in "${EXAMPLE_FILES[@]}"; do
    SOURCE_FILE="${EXAMPLES_SOURCE}/${file}"
    TARGET_FILE="${EXAMPLES_TARGET}/${file}"

    if [ ! -f "${SOURCE_FILE}" ]; then
        echo -e "${YELLOW}경고: ${file} 파일을 찾을 수 없습니다. 건너뜁니다.${NC}"
        SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
        continue
    fi

    # 기존 파일이 있는지 확인
    if [ -f "${TARGET_FILE}" ]; then
        echo -e "${YELLOW}파일이 이미 존재합니다: ${file}${NC}"
        read -p "덮어쓰시겠습니까? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "건너뜁니다: ${file}"
            SKIPPED_COUNT=$((SKIPPED_COUNT + 1))
            continue
        fi
    fi

    # 파일 복사
    cp "${SOURCE_FILE}" "${TARGET_FILE}"
    echo -e "${GREEN}✓ 복사 완료: ${file}${NC}"
    COPIED_COUNT=$((COPIED_COUNT + 1))
done

echo ""
echo "================================"
echo "설치 완료!"
echo "  - 복사됨: ${COPIED_COUNT}개"
echo "  - 건너뜀: ${SKIPPED_COUNT}개"
echo ""

# 실행 방법 안내
if [ ${COPIED_COUNT} -gt 0 ]; then
    echo "예제 실행 방법:"
    echo "  cd ${PROJECT_ROOT}/rust/crates/mecab-ko-core"
    echo ""
    for file in "${EXAMPLE_FILES[@]}"; do
        example_name="${file%.rs}"
        echo "  cargo run --example ${example_name}"
    done
    echo ""

    # 사전 설치 확인
    if ! command -v mecab-config &> /dev/null; then
        echo -e "${YELLOW}주의: MeCab-Ko 사전이 설치되지 않은 것 같습니다.${NC}"
        echo ""
        echo "사전 설치 방법:"
        echo "  Ubuntu/Debian: sudo apt-get install mecab-ko mecab-ko-dic"
        echo "  macOS: brew install mecab-ko mecab-ko-dic"
        echo ""
    else
        DICDIR=$(mecab-config --dicdir 2>/dev/null || echo "unknown")
        echo -e "${GREEN}✓ MeCab-Ko 사전 경로: ${DICDIR}${NC}"
    fi
fi

exit 0
