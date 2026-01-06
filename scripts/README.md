# MeCab-Ko 말뭉치 처리 스크립트

한국어 말뭉치를 MeCab 사전 형식으로 변환하고 처리하는 Python 스크립트 모음입니다.

## 목차

- [개요](#개요)
- [요구사항](#요구사항)
- [스크립트 설명](#스크립트-설명)
- [사용법](#사용법)
- [지원 형식](#지원-형식)
- [예제](#예제)
- [저작권 및 라이선스](#저작권-및-라이선스)

## 개요

이 스크립트들은 다양한 한국어 말뭉치 형식을 MeCab-Ko 사전 형식으로 변환하고, 신조어를 추출하며, 여러 사전을 병합하는 기능을 제공합니다.

### 주요 기능

- **말뭉치 → 사전 변환**: 모두의 말뭉치, 세종 말뭉치, CoNLL-U 형식 지원
- **신조어 추출**: 패턴 기반 신조어 탐지 및 추출
- **사전 병합**: 여러 사전 파일을 병합하고 중복 제거
- **통계 분석**: 품사 분포, 빈도 분석, 길이 분포 등

## 요구사항

### 필수 요구사항

- Python 3.10 이상
- Python 표준 라이브러리 (추가 패키지 불필요)

### 선택적 패키지

추가 기능을 위해 다음 패키지를 설치할 수 있습니다:

```bash
# 기본 설치 (선택사항)
pip install -r requirements.txt

# 또는 필요한 패키지만 선택적으로 설치
pip install tqdm  # 진행률 표시
pip install pandas numpy  # 데이터 분석
```

## 스크립트 설명

### 1. corpus_to_dict.py

말뭉치 파일에서 MeCab 사전 엔트리를 추출합니다.

**특징:**
- 2-pass 처리: 빈도 수집 → 엔트리 생성
- 자동 비용(cost) 계산 (빈도 기반)
- 품사 태그 매핑
- 통계 분석 및 보고

**지원 형식:**
- 모두의 말뭉치 (JSON)
- 세종 말뭉치 (XML)
- CoNLL-U

### 2. extract_neologisms.py

말뭉치에서 신조어를 자동으로 추출합니다.

**탐지 패턴:**
- 외래어/영어 혼용 (예: "먹방", "셀카")
- 축약어 (예: "강추", "별다줄")
- 접두사/접미사 패턴 (예: "극혐", "~질")
- 반복 패턴 (예: "쩝쩝", "방방")
- 이모티콘/의태어 (예: "ㅋㅋ", "ㅠㅠ")

**필터링:**
- 빈도 기반 필터링 (최소/최대 빈도)
- 기존 사전 제외 (선택적)
- 길이 필터링
- 일반 단어 블랙리스트

### 3. merge_dictionaries.py

여러 MeCab 사전 CSV 파일을 병합합니다.

**충돌 해결 전략:**
- `min_cost`: 최소 비용 선택 (기본값)
- `max_cost`: 최대 비용 선택
- `first`: 첫 번째 엔트리 유지
- `last`: 마지막 엔트리 유지
- `avg_cost`: 비용 평균 계산

**기능:**
- 자동 중복 제거
- 사전 통계 분석
- 품사/빈도 분포 분석

## 사용법

### 기본 사용법

모든 스크립트는 실행 권한을 부여하고 직접 실행하거나 Python으로 실행할 수 있습니다:

```bash
# 실행 권한 부여
chmod +x corpus_to_dict.py extract_neologisms.py merge_dictionaries.py

# 직접 실행
./corpus_to_dict.py --help

# 또는 Python으로 실행
python3 corpus_to_dict.py --help
```

### corpus_to_dict.py

```bash
# 모두의 말뭉치 변환
python3 corpus_to_dict.py \
  -f modu \
  -i /path/to/modu/corpus/ \
  -o output/modu_dict.csv

# 최소 빈도 필터 적용
python3 corpus_to_dict.py \
  -f modu \
  -i corpus/modu/ \
  -o output/modu_dict.csv \
  --min-freq 3

# 세종 말뭉치 변환
python3 corpus_to_dict.py \
  -f sejong \
  -i corpus/sejong.xml \
  -o output/sejong_dict.csv

# CoNLL-U 형식 변환
python3 corpus_to_dict.py \
  -f conllu \
  -i corpus/data.conllu \
  -o output/conllu_dict.csv \
  -v  # 상세 로그
```

### extract_neologisms.py

```bash
# 신조어 추출 (JSON 출력)
python3 extract_neologisms.py \
  -f modu \
  -i corpus/modu/ \
  -o output/neologisms.json

# CSV 출력 형식
python3 extract_neologisms.py \
  -f modu \
  -i corpus/modu/ \
  -o output/neologisms.csv \
  --output-format csv

# 기존 사전 제외
python3 extract_neologisms.py \
  -f modu \
  -i corpus/modu/ \
  -o output/neologisms.json \
  --reference-dict /path/to/existing_dict.csv

# 빈도 및 길이 필터 조정
python3 extract_neologisms.py \
  -f modu \
  -i corpus/modu/ \
  -o output/neologisms.json \
  --min-freq 5 \
  --max-freq 500 \
  --min-length 2 \
  --max-length 8
```

### merge_dictionaries.py

```bash
# 여러 사전 병합
python3 merge_dictionaries.py \
  -i dict1.csv dict2.csv dict3.csv \
  -o merged.csv

# 최소 비용 전략 (기본값)
python3 merge_dictionaries.py \
  -i modu_dict.csv sejong_dict.csv neologism_dict.csv \
  -o merged_final.csv \
  --strategy min_cost

# 중복 제거 없이 병합
python3 merge_dictionaries.py \
  -i dict1.csv dict2.csv \
  -o merged.csv \
  --no-deduplicate

# 사전 통계 분석만 수행
python3 merge_dictionaries.py \
  --analyze existing_dict.csv
```

## 지원 형식

### 입력 형식

#### 1. 모두의 말뭉치 (Modu Corpus) - JSON

```json
{
  "document": [
    {
      "sentence": [
        {
          "word": [
            {
              "form": "나는",
              "morpheme": [
                {"form": "나", "label": "NP"},
                {"form": "는", "label": "JX"}
              ]
            }
          ]
        }
      ]
    }
  ]
}
```

#### 2. 세종 말뭉치 (Sejong Corpus) - XML

```xml
<corpus>
  <sentence>
    <word>
      <morph tag="NP">나</morph>
      <morph tag="JX">는</morph>
    </word>
  </sentence>
</corpus>
```

#### 3. CoNLL-U 형식

```
# sent_id = 1
1	나	나	NP	_	_	_	_	_	_
2	는	는	JX	_	_	_	_	_	_

# sent_id = 2
...
```

### 출력 형식

#### MeCab CSV 사전 형식

```csv
surface,left_id,right_id,cost,pos,pos_detail1,pos_detail2,pos_detail3,inflection_type,inflection_form,base_form,reading,pronunciation
나,0,0,3500,NP,*,*,*,*,*,나,*,*
는,0,0,2800,JX,*,*,*,*,*,는,*,*
```

**필드 설명:**
1. `surface`: 표층형 (실제 단어)
2. `left_id`: 좌측 문맥 ID (mecab-dict-index에서 설정)
3. `right_id`: 우측 문맥 ID
4. `cost`: 비용 (낮을수록 우선순위 높음)
5. `pos`: 품사 (주 품사 태그)
6-8. `pos_detail1-3`: 품사 세부 정보
9-10. `inflection_type/form`: 활용 유형/형태
11. `base_form`: 기본형
12. `reading`: 읽기
13. `pronunciation`: 발음

## 예제

### 전체 워크플로우 예제

```bash
#!/bin/bash
# MeCab-Ko 사전 구축 전체 워크플로우

# 1. 모두의 말뭉치에서 기본 사전 생성
echo "Step 1: Converting Modu corpus..."
python3 corpus_to_dict.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/modu_base.csv \
  --min-freq 2

# 2. 세종 말뭉치 추가
echo "Step 2: Converting Sejong corpus..."
python3 corpus_to_dict.py \
  -f sejong \
  -i /data/corpus/sejong/ \
  -o output/sejong_base.csv \
  --min-freq 2

# 3. 신조어 추출
echo "Step 3: Extracting neologisms..."
python3 extract_neologisms.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/neologisms.json \
  --reference-dict output/modu_base.csv \
  --min-freq 3 \
  --max-freq 100

# 4. 신조어를 CSV로 변환 (수동 검토용)
python3 extract_neologisms.py \
  -f modu \
  -i /data/corpus/modu/ \
  -o output/neologisms_review.csv \
  --output-format csv \
  --reference-dict output/modu_base.csv

# 5. 검토 후 승인된 신조어와 기존 사전 병합
echo "Step 4: Merging dictionaries..."
python3 merge_dictionaries.py \
  -i output/modu_base.csv \
     output/sejong_base.csv \
     output/neologisms_approved.csv \
  -o output/final_dict.csv \
  --strategy min_cost

# 6. 최종 사전 분석
echo "Step 5: Analyzing final dictionary..."
python3 merge_dictionaries.py \
  --analyze output/final_dict.csv

echo "Done! Final dictionary: output/final_dict.csv"
```

### Python 스크립트에서 활용

```python
#!/usr/bin/env python3
"""사용자 정의 처리 예제"""

import csv
from pathlib import Path
from corpus_to_dict import ModuCorpusParser, DictEntry

# 커스텀 파서 생성
parser = ModuCorpusParser(min_frequency=3)

# 말뭉치 파싱
entries = list(parser.parse(Path("/data/corpus/modu/")))

# 특정 품사만 필터링
nouns = [e for e in entries if e.pos.startswith("NN")]

# CSV 출력
output_path = Path("output/nouns_only.csv")
with open(output_path, "w", encoding="utf-8", newline="") as f:
    writer = csv.writer(f)
    for entry in nouns:
        writer.writerow(entry.to_csv_row())

print(f"Extracted {len(nouns):,} nouns to {output_path}")
```

## 성능 및 최적화

### 처리 속도

- **corpus_to_dict.py**: ~10,000 문장/초 (JSON)
- **extract_neologisms.py**: ~5,000 문장/초
- **merge_dictionaries.py**: ~100,000 엔트리/초

### 메모리 사용

- 대용량 말뭉치의 경우 2-pass 처리로 메모리 효율성 확보
- 통계 정보는 메모리에 유지 (Counter 사용)

### 대용량 말뭉치 처리 팁

```bash
# 디렉토리를 여러 배치로 나누어 처리
for dir in corpus/batch_*; do
  python3 corpus_to_dict.py \
    -f modu \
    -i "$dir" \
    -o "output/$(basename $dir).csv" \
    --min-freq 2
done

# 결과 병합
python3 merge_dictionaries.py \
  -i output/batch_*.csv \
  -o output/final.csv
```

## 저작권 및 라이선스

### 스크립트 라이선스

이 스크립트들은 MeCab-Ko 프로젝트의 일부로 다음 라이선스를 따릅니다:

- **MeCab-Ko**: GPL, LGPL, BSD 3중 라이선스

### 말뭉치 저작권 주의사항

⚠️ **중요**: 한국어 말뭉치 사용 시 반드시 각 말뭉치의 라이선스를 확인하세요.

#### 1. 모두의 말뭉치

- **제공**: 국립국어원
- **라이선스**: CC BY-SA 2.0 KR
- **사용 조건**:
  - 상업적 사용 가능
  - 2차 저작물 작성 가능
  - 동일 조건 공유 (Share-Alike) 필수
  - 출처 표시 필수

**출처 표시 예시:**
```
이 자료는 국립국어원의 "모두의 말뭉치" 자료를 활용하였습니다.
(https://corpus.korean.go.kr)
```

#### 2. 세종 말뭉치

- **제공**: 국립국어원
- **라이선스**: 연구 목적 사용 제한
- **사용 조건**:
  - 학술 연구 목적으로만 사용 가능
  - 상업적 사용 전 별도 승인 필요
  - 재배포 제한

⚠️ **주의**: 세종 말뭉치로 생성한 사전을 상업적으로 배포하기 전에 국립국어원의 승인을 받아야 합니다.

#### 3. 사용자 수집 말뭉치

자체 수집한 말뭉치 사용 시 주의사항:

- 웹 크롤링 데이터: robots.txt 및 이용약관 준수
- SNS 데이터: 개인정보 보호법 준수
- 저작권이 있는 텍스트: 저작권자 동의 필요

### 생성된 사전의 라이선스

생성된 MeCab 사전의 라이선스는 **원본 말뭉치의 라이선스를 따릅니다**:

- 모두의 말뭉치 기반 → CC BY-SA 2.0 KR
- 세종 말뭉치 기반 → 연구 목적 제한
- 혼합 사전 → 가장 제한적인 라이선스 적용

### 권장 사항

1. **라이선스 문서화**: 사전 배포 시 사용한 말뭉치 목록과 라이선스 명시
2. **출처 표시**: README 또는 LICENSES 파일에 출처 기록
3. **상업적 사용**: 변호사 또는 라이선스 전문가와 상담

### 라이선스 파일 예제

```markdown
# Dictionary License Information

This dictionary was created using the following corpora:

## Modu Corpus (모두의 말뭉치)
- Source: National Institute of Korean Language
- License: CC BY-SA 2.0 KR
- URL: https://corpus.korean.go.kr
- Usage: 70% of entries

## Custom Corpus
- Source: Internal collection
- License: Proprietary
- Usage: 30% of entries

The resulting dictionary is licensed under CC BY-SA 2.0 KR
due to the share-alike requirement of the Modu Corpus.
```

## 문제 해결

### 일반적인 문제

**1. UnicodeDecodeError**
```bash
# UTF-8 인코딩 확인
file -i your_corpus.json

# 인코딩 변환이 필요한 경우
iconv -f EUC-KR -t UTF-8 input.txt > output.txt
```

**2. 메모리 부족**
```bash
# 배치 처리 사용 (위 "대용량 말뭉치 처리 팁" 참조)
```

**3. JSON 파싱 오류**
```bash
# JSON 유효성 검사
python3 -m json.tool your_file.json > /dev/null
```

## 기여하기

버그 리포트, 기능 제안, 코드 기여는 GitHub 이슈 및 PR로 환영합니다.

## 참고 자료

- [MeCab 공식 문서](https://taku910.github.io/mecab/)
- [MeCab-Ko 프로젝트](https://bitbucket.org/eunjeon/mecab-ko)
- [모두의 말뭉치](https://corpus.korean.go.kr)
- [국립국어원](https://korean.go.kr)

## 연락처

- 이슈: GitHub Issues
- 이메일: (프로젝트 관리자 이메일)

---

**최종 업데이트**: 2026-01-05
