# 빠른 시작 가이드

MeCab-Ko 말뭉치 처리 스크립트를 5분 안에 시작하는 방법입니다.

## 1단계: 환경 준비 (30초)

```bash
cd /home/mare/mecab-ko/scripts

# 실행 권한 부여
chmod +x *.py

# Python 버전 확인 (3.10 이상 필요)
python3 --version
```

## 2단계: 샘플 데이터로 테스트 (2분)

### 샘플 말뭉치 생성

```bash
# 테스트용 샘플 JSON 생성
cat > sample_modu.json << 'EOF'
{
  "document": [
    {
      "id": "DOC001",
      "sentence": [
        {
          "id": 1,
          "form": "자연어 처리는 재미있다.",
          "word": [
            {
              "id": 1,
              "form": "자연어",
              "morpheme": [
                {"id": 1, "form": "자연", "label": "NNG"},
                {"id": 2, "form": "어", "label": "NNG"}
              ]
            },
            {
              "id": 2,
              "form": "처리는",
              "morpheme": [
                {"id": 1, "form": "처리", "label": "NNG"},
                {"id": 2, "form": "는", "label": "JX"}
              ]
            },
            {
              "id": 3,
              "form": "재미있다",
              "morpheme": [
                {"id": 1, "form": "재미있", "label": "VA"},
                {"id": 2, "form": "다", "label": "EF"}
              ]
            },
            {
              "id": 4,
              "form": ".",
              "morpheme": [
                {"id": 1, "form": ".", "label": "SF"}
              ]
            }
          ]
        },
        {
          "id": 2,
          "form": "MeCab은 빠른 형태소 분석기이다.",
          "word": [
            {
              "id": 1,
              "form": "MeCab",
              "morpheme": [
                {"id": 1, "form": "MeCab", "label": "SL"}
              ]
            },
            {
              "id": 2,
              "form": "은",
              "morpheme": [
                {"id": 1, "form": "은", "label": "JX"}
              ]
            },
            {
              "id": 3,
              "form": "빠른",
              "morpheme": [
                {"id": 1, "form": "빠르", "label": "VA"},
                {"id": 2, "form": "ㄴ", "label": "ETM"}
              ]
            },
            {
              "id": 4,
              "form": "형태소",
              "morpheme": [
                {"id": 1, "form": "형태", "label": "NNG"},
                {"id": 2, "form": "소", "label": "NNG"}
              ]
            },
            {
              "id": 5,
              "form": "분석기이다",
              "morpheme": [
                {"id": 1, "form": "분석", "label": "NNG"},
                {"id": 2, "form": "기", "label": "NNG"},
                {"id": 3, "form": "이", "label": "VCP"},
                {"id": 4, "form": "다", "label": "EF"}
              ]
            },
            {
              "id": 6,
              "form": ".",
              "morpheme": [
                {"id": 1, "form": ".", "label": "SF"}
              ]
            }
          ]
        }
      ]
    }
  ]
}
EOF

# 출력 디렉토리 생성
mkdir -p output
```

### 말뭉치 → 사전 변환

```bash
./corpus_to_dict.py \
  -f modu \
  -i sample_modu.json \
  -o output/sample_dict.csv \
  -v
```

**예상 출력:**
```
2026-01-05 10:00:00 - INFO - Parsing Modu corpus: sample_modu.json
2026-01-05 10:00:00 - INFO - First pass: collecting frequency statistics...
2026-01-05 10:00:00 - INFO - Second pass: generating dictionary entries...
2026-01-05 10:00:01 - INFO - Generated 15 dictionary entries
2026-01-05 10:00:01 - INFO - Writing dictionary to: output/sample_dict.csv
2026-01-05 10:00:01 - INFO - Successfully wrote 15 entries to output/sample_dict.csv
```

### 결과 확인

```bash
# 처음 5줄 확인
head -5 output/sample_dict.csv

# 전체 라인 수
wc -l output/sample_dict.csv
```

## 3단계: 실제 말뭉치 사용 (2분)

### 모두의 말뭉치 다운로드

1. [모두의 말뭉치](https://corpus.korean.go.kr) 방문
2. 회원가입 (무료)
3. 원하는 말뭉치 다운로드 (예: 신문 말뭉치, 메신저 말뭉치 등)
4. 압축 해제

### 실제 말뭉치 변환

```bash
# 예: 다운로드한 말뭉치가 /data/modu/에 있다고 가정
./corpus_to_dict.py \
  -f modu \
  -i /data/modu/ \
  -o output/modu_full.csv \
  --min-freq 2 \
  -v
```

## 일반적인 워크플로우

### 시나리오 1: 기본 사전 만들기

```bash
# 1. 말뭉치 변환
./corpus_to_dict.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/base_dict.csv \
  --min-freq 2

# 2. 사전 통계 확인
./merge_dictionaries.py --analyze output/base_dict.csv
```

### 시나리오 2: 신조어 추출하기

```bash
# 1. 기본 사전이 있어야 함 (위 시나리오 1 참조)

# 2. 신조어 추출
./extract_neologisms.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/neologisms.json \
  --reference-dict output/base_dict.csv \
  --min-freq 3 \
  --max-freq 100

# 3. CSV로도 추출 (검토용)
./extract_neologisms.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/neologisms.csv \
  --output-format csv \
  --reference-dict output/base_dict.csv \
  --min-freq 3
```

### 시나리오 3: 여러 사전 병합하기

```bash
# 여러 소스의 사전 병합
./merge_dictionaries.py \
  -i output/modu_dict.csv \
     output/sejong_dict.csv \
     output/custom_dict.csv \
  -o output/merged_final.csv \
  --strategy min_cost \
  -v
```

## 고급 사용법

### 대용량 말뭉치 배치 처리

```bash
#!/bin/bash
# batch_process.sh

INPUT_DIR="/data/corpus/large_corpus"
OUTPUT_DIR="output/batches"
mkdir -p "$OUTPUT_DIR"

# 배치 크기로 나누어 처리
find "$INPUT_DIR" -name "*.json" | split -l 100 - batch_list_

for batch_file in batch_list_*; do
  batch_num=$(echo $batch_file | sed 's/batch_list_//')
  echo "Processing batch $batch_num..."

  # 배치별로 처리
  cat $batch_file | while read json_file; do
    ./corpus_to_dict.py \
      -f modu \
      -i "$json_file" \
      -o "${OUTPUT_DIR}/batch_${batch_num}.csv" \
      --min-freq 1
  done
done

# 모든 배치 병합
./merge_dictionaries.py \
  -i ${OUTPUT_DIR}/batch_*.csv \
  -o output/final_large_dict.csv

# 정리
rm -rf batch_list_*
```

### 품사별 필터링

```python
#!/usr/bin/env python3
"""특정 품사만 추출"""

import csv
from pathlib import Path

def filter_by_pos(input_csv, output_csv, pos_prefixes):
    """품사 접두사로 필터링"""
    with open(input_csv, encoding='utf-8') as fin:
        with open(output_csv, 'w', encoding='utf-8', newline='') as fout:
            reader = csv.reader(fin)
            writer = csv.writer(fout)

            for row in reader:
                if len(row) >= 5:
                    pos = row[4]  # POS 필드
                    if any(pos.startswith(prefix) for prefix in pos_prefixes):
                        writer.writerow(row)

# 명사만 추출
filter_by_pos(
    'output/base_dict.csv',
    'output/nouns_only.csv',
    ['NN']  # NNG, NNP, NNB
)

# 동사+형용사 추출
filter_by_pos(
    'output/base_dict.csv',
    'output/verbs_adjectives.csv',
    ['VV', 'VA']
)
```

## 트러블슈팅

### 문제 1: "ModuleNotFoundError"

```bash
# Python 버전 확인
python3 --version  # 3.10 이상이어야 함

# Python 경로 확인
which python3

# 필요시 가상환경 생성
python3 -m venv venv
source venv/bin/activate
```

### 문제 2: "UnicodeDecodeError"

```bash
# 파일 인코딩 확인
file -i your_file.json

# UTF-8로 변환
iconv -f EUC-KR -t UTF-8 input.json > output_utf8.json
```

### 문제 3: "MemoryError"

```bash
# 배치 처리 사용 (위 "대용량 말뭉치 배치 처리" 참조)

# 또는 최소 빈도 높이기
./corpus_to_dict.py \
  -f modu \
  -i /data/large_corpus/ \
  -o output/dict.csv \
  --min-freq 5  # 빈도 제한 상향
```

### 문제 4: JSON 파싱 오류

```bash
# JSON 유효성 검사
python3 -m json.tool problematic.json > /dev/null

# 오류가 있으면 수정 또는 제외
```

## 다음 단계

### MeCab 사전으로 컴파일

생성된 CSV를 MeCab 바이너리 사전으로 컴파일:

```bash
# mecab-dict-index 사용
/usr/local/libexec/mecab/mecab-dict-index \
  -d /usr/local/lib/mecab/dic/mecab-ko-dic \
  -u output/custom.dic \
  -f utf-8 \
  -t utf-8 \
  output/final_dict.csv

# 사전 테스트
echo "자연어 처리는 재미있다" | mecab -u output/custom.dic
```

### Rust 구현과 통합

```bash
# Rust mecab-ko-dict 크레이트에서 사용
cd /home/mare/mecab-ko/rust/mecab-ko-dict

# CSV를 Rust 바이너리 포맷으로 변환 (TODO: 구현 예정)
cargo run --bin dict-compiler -- \
  --input /home/mare/mecab-ko/scripts/output/final_dict.csv \
  --output dict.bin
```

## 유용한 명령어 모음

```bash
# 사전 엔트리 수 세기
wc -l output/*.csv

# 특정 단어 찾기
grep "^자연어," output/dict.csv

# 품사 분포 확인
cut -d',' -f5 output/dict.csv | sort | uniq -c | sort -rn | head -20

# 가장 비용이 낮은 단어 (우선순위 높음)
sort -t',' -k4 -n output/dict.csv | head -20

# 가장 긴 단어
awk -F',' '{print length($1), $1}' output/dict.csv | sort -rn | head -20

# 특정 품사만 카운트
grep ",NNG," output/dict.csv | wc -l
```

## 추가 리소스

- [상세 README](README.md)
- [저작권 가이드](CORPUS_LICENSES.md)
- [MeCab-Ko 문서](/home/mare/mecab-ko/docs/)
- [GitHub 이슈](https://github.com/your-repo/issues)

## 도움이 필요하신가요?

```bash
# 각 스크립트의 도움말 보기
./corpus_to_dict.py --help
./extract_neologisms.py --help
./merge_dictionaries.py --help
```

---

**최소 실행 시간**: 5분
**난이도**: 초급
**최종 업데이트**: 2026-01-05
