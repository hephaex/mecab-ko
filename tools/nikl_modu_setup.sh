#!/bin/bash
# NIKL Modu 다운로드 후 원샷 변환 + 평가 스크립트.
#
# 사용법:
#   ./tools/nikl_modu_setup.sh <path/to/NXMP*.json>
#   ./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP1902008051.json
#
# 동작:
#   1. JSON 파일 존재/유효성 확인
#   2. python3 tools/convert_nikl_modu.py 실행 → data/eval/nikl_modu_sample.tsv
#   3. cargo test test_nikl_modu_dual_metric 실행 → 정확도 측정
#   4. 결과 요약 출력

set -e

INPUT_JSON="${1:-}"
OUTPUT_TSV="data/eval/nikl_modu_sample.tsv"
MAX_SENTENCES="${MAX_SENTENCES:-5000}"

# 1. 입력 확인
if [ -z "$INPUT_JSON" ]; then
    echo "ERROR: NIKL Modu JSON 파일 경로 필수"
    echo ""
    echo "사용법:"
    echo "  ./tools/nikl_modu_setup.sh <path/to/NXMP*.json>"
    echo ""
    echo "다운로드 방법:"
    echo "  1. https://kli.korean.go.kr 학술 등록 (1-3일 승인)"
    echo "  2. '모두의말뭉치 형태분석' 다운로드"
    echo "  3. JSON 파일을 임의 경로에 배치"
    echo ""
    echo "자세한 설정: docs/eval/nikl_modu_setup.md"
    exit 1
fi

if [ ! -f "$INPUT_JSON" ]; then
    echo "ERROR: 파일을 찾을 수 없음: $INPUT_JSON"
    exit 1
fi

# 파일 크기 체크 (NIKL Modu JSON은 보통 수십 MB 이상)
SIZE_BYTES=$(stat -f%z "$INPUT_JSON" 2>/dev/null || stat -c%s "$INPUT_JSON" 2>/dev/null)
SIZE_MB=$((SIZE_BYTES / 1024 / 1024))
if [ "$SIZE_MB" -lt 1 ]; then
    echo "WARNING: 파일이 매우 작음 (${SIZE_MB} MB). 올바른 NIKL Modu 파일인지 확인하세요."
fi
echo "✓ JSON 파일: $INPUT_JSON (${SIZE_MB} MB)"

# 2. 변환
echo ""
echo "=== Step 1: JSON → TSV 변환 ==="
python3 tools/convert_nikl_modu.py "$INPUT_JSON" "$OUTPUT_TSV" --max-sentences "$MAX_SENTENCES"

if [ ! -f "$OUTPUT_TSV" ]; then
    echo "ERROR: TSV 생성 실패"
    exit 1
fi
TSV_LINES=$(wc -l < "$OUTPUT_TSV")
echo "✓ TSV: $OUTPUT_TSV ($TSV_LINES sentences)"

# 3. 평가
echo ""
echo "=== Step 2: 정확도 측정 ==="
cd rust
cargo test --package mecab-ko-core --test accuracy_eval \
    -- test_nikl_modu_dual_metric --nocapture --ignored 2>&1 \
    | grep -E "===|Morpheme:|Eojeol:|PASSED|FAILED"

# 4. 결과 요약
echo ""
echo "=== 완료 ==="
echo "다음 단계:"
echo "  - 결과를 PROGRESS.md에 기록"
echo "  - POS mismatch 분석 (필요 시)"
echo "  - 추가 동치/normalize 후보 발굴"
