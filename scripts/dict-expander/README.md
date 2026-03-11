# MeCab-Ko Dictionary Expander

사전 범위 확장을 위한 Python 기반 도구 모음입니다. 다양한 데이터 소스로부터 자동으로 MeCab 사전 항목을 생성합니다.

## 개요

이 도구는 MeCab-Ko 사전의 범위를 확장하기 위해 다음 기능을 제공합니다:

- **고유명사 확장**: 인명, 지명, 기관명 등의 고유명사 자동 추출
- **복합명사 생성**: 기존 명사 조합으로 복합명사 생성
- **활용형 생성**: 동사/형용사의 다양한 활용형 생성
- **약어 처리**: 줄임말 및 두문자어 자동 생성

## 디렉토리 구조

```
dict-expander/
├── expand_proper_nouns.py      # 고유명사 확장 도구
├── expand_compounds.py          # 복합명사 생성 도구
├── expand_conjugations.py       # 활용형 생성 도구
├── expand_abbreviations.py      # 약어 확장 도구
├── data_sources/                # 데이터 소스 모듈
│   ├── wikipedia_fetcher.py    # 위키피디아 데이터
│   └── public_data_fetcher.py  # 공공데이터 API
├── validators/                  # 검증 모듈
│   ├── deduplicator.py         # 중복 제거
│   ├── pos_inference.py        # 품사 태그 추론
│   └── quality_checker.py      # 품질 검증
└── utils/                       # 유틸리티 모듈
    ├── mecab_format.py         # MeCab 포맷 처리
    └── korean_utils.py         # 한글 처리 유틸리티
```

## 설치

### 요구사항

- Python 3.10 이상
- 표준 라이브러리만 사용 (외부 의존성 없음)

### 선택적 의존성

추가 기능을 위한 선택적 패키지:

```bash
pip install -r requirements.txt
```

## 사용법

### 1. 고유명사 확장 (expand_proper_nouns.py)

위키피디아, 공공데이터 등에서 고유명사를 추출합니다.

#### Wikipedia에서 배우 이름 추출

```bash
python expand_proper_nouns.py \
    --source wikipedia \
    --category "대한민국의_배우" \
    --type person \
    -o person_actors.csv
```

#### 공공데이터에서 정부기관명 추출

```bash
python expand_proper_nouns.py \
    --source public_data \
    --type organization \
    -o organizations.csv
```

#### 텍스트 파일에서 지명 추출

```bash
# places.txt에 한 줄당 하나씩 지명 입력
python expand_proper_nouns.py \
    --input places.txt \
    --type place \
    -o place_names.csv
```

#### 옵션

- `--source`: 데이터 소스 (`wikipedia`, `public_data`)
- `--input`: 입력 텍스트 파일 (한 줄당 하나의 고유명사)
- `--category`: 위키피디아 카테고리 필터
- `--type`: 고유명사 유형 (`person`, `place`, `organization`, `proper`)
- `--limit`: 최대 생성 항목 수
- `-o, --output`: 출력 CSV 파일 (필수)
- `--no-validate`: 검증 비활성화
- `--no-deduplicate`: 중복 제거 비활성화

### 2. 복합명사 생성 (expand_compounds.py)

기존 명사들을 조합하여 복합명사를 생성합니다.

#### 사전에서 복합명사 생성

```bash
python expand_compounds.py \
    --dict /path/to/mecab-ko-dic/seed \
    --combine \
    --max-components 2 \
    -o compounds.csv
```

#### 접미사를 사용한 복합명사 생성

```bash
python expand_compounds.py \
    --input base_nouns.txt \
    --suffixes \
    -o compound_suffixes.csv
```

#### 패턴 파일을 사용한 생성

```bash
# patterns.txt 형식:
# 컴퓨터 공학
# 인공 지능
# 자연 언어 처리

python expand_compounds.py \
    --dict /path/to/dict \
    --patterns patterns.txt \
    -o pattern_compounds.csv
```

#### 옵션

- `--dict`: 기본 형태소가 있는 사전 디렉토리
- `--input`: 기본 명사 텍스트 파일
- `--patterns`: 패턴 파일 (공백으로 구분된 형태소)
- `--combine`: 모든 조합 생성 (주의: 많은 항목 생성)
- `--suffixes`: 공통 접미사로 복합명사 생성
- `--max-components`: 복합명사당 최대 형태소 수 (기본값: 2)
- `--max-length`: 최대 음절 길이 (기본값: 10)
- `-o, --output`: 출력 CSV 파일 (필수)

### 3. 활용형 생성 (expand_conjugations.py)

동사와 형용사의 다양한 활용형을 생성합니다.

#### 사전에서 활용형 생성

```bash
python expand_conjugations.py \
    --dict /path/to/VV.csv \
    --conjugate \
    --patterns common \
    -o conjugations.csv
```

#### 텍스트 파일에서 동사 활용형 생성

```bash
# verbs.txt에 한 줄당 하나씩 동사 (예: 하다, 가다, 먹다)
python expand_conjugations.py \
    --input verbs.txt \
    --patterns present_informal past connecting \
    -o verb_conjugations.csv
```

#### 옵션

- `--dict`: 동사/형용사 CSV 파일
- `--input`: 동사 텍스트 파일 (한 줄당 하나)
- `--conjugate`: 일반적인 활용형 생성
- `--patterns`: 활용 패턴 선택
  - `present_informal`: 현재 비격식 (-아/-어/-여)
  - `past`: 과거 (-았/-었/-였)
  - `future`: 미래/추측 (-겠)
  - `honorific`: 높임 (-시)
  - `connecting`: 연결 (-고/-며/-면/-지만)
  - `common`: 일반적인 패턴 (present_informal + past + connecting)
- `--no-irregular`: 불규칙 활용형 생성 안 함
- `-o, --output`: 출력 CSV 파일 (필수)

### 4. 약어 확장 (expand_abbreviations.py)

줄임말과 두문자어를 자동으로 생성합니다.

#### 복합명사에서 약어 추출

```bash
python expand_abbreviations.py \
    --dict compounds.csv \
    --extract-initials \
    -o abbreviations.csv
```

#### 약어 매핑 파일 사용

```bash
# abbrev_map.txt 형식:
# KBS=한국방송공사
# MBC=문화방송
# SBS=서울방송

python expand_abbreviations.py \
    --map abbrev_map.txt \
    -o initialisms.csv
```

#### 옵션

- `--dict`: 복합명사가 있는 사전 CSV
- `--input`: 입력 텍스트 파일
- `--map`: 약어 매핑 파일 (`abbrev=full_form` 형식)
- `--extract-initials`: 복합명사에서 두문자 추출
- `--patterns`: 약어 패턴 (`korean`, `english`, `both`)
- `--min-length`: 최소 약어 길이 (기본값: 2)
- `--max-length`: 최대 약어 길이 (기본값: 6)
- `-o, --output`: 출력 CSV 파일 (필수)

## 출력 형식

모든 도구는 MeCab CSV 형식으로 출력합니다:

```csv
surface,left_id,right_id,cost,pos,semantic,has_jongseong,reading,type,first_pos,last_pos,expression
서울,0,0,0,NNP,지명,T,서울,*,*,*,*
컴퓨터공학,0,0,0,NNG,*,T,컴퓨터공학,Compound,NNG,NNG,컴퓨터/NNG/*+공학/NNG/*
```

### 필드 설명

- `surface`: 표층형
- `left_id`, `right_id`, `cost`: 연결 비용 정보 (기본값: 0)
- `pos`: 품사 태그 (NNG, NNP, VV, VA 등)
- `semantic`: 의미 분류 (인명, 지명, 기관, 약어 등)
- `has_jongseong`: 종성 유무 (T/F)
- `reading`: 읽기
- `type`: 항목 유형 (Compound, Inflect, Preanalysis 등)
- `first_pos`, `last_pos`: 복합어의 첫/마지막 품사
- `expression`: 형태소 분석 표현

## 통합 워크플로우

여러 도구를 조합하여 사용하는 예제:

```bash
#!/bin/bash

# 1. 위키피디아에서 배우 이름 추출
python expand_proper_nouns.py \
    --source wikipedia \
    --category "대한민국의_배우" \
    --type person \
    -o output/person_actors.csv

# 2. 공공데이터에서 지명 추출
python expand_proper_nouns.py \
    --source public_data \
    --type place \
    -o output/places.csv

# 3. 복합명사 생성
python expand_compounds.py \
    --dict /path/to/mecab-ko-dic/seed \
    --combine \
    --max-components 2 \
    -o output/compounds.csv

# 4. 복합명사에서 약어 생성
python expand_abbreviations.py \
    --dict output/compounds.csv \
    --extract-initials \
    -o output/abbreviations.csv

# 5. 동사 활용형 생성
python expand_conjugations.py \
    --dict /path/to/VV.csv \
    --patterns common \
    -o output/conjugations.csv

# 6. 모든 출력 병합
cat output/*.csv > expanded_dictionary.csv

# 7. 기존 사전과 통합 (scripts 디렉토리 도구 사용)
cd ../../scripts
python merge_dictionaries.py \
    --base /path/to/mecab-ko-dic/seed \
    --additional ../tools/dict-expander/expanded_dictionary.csv \
    --output /path/to/output/merged_dict
```

## 품질 검증

모든 도구는 내장된 품질 검증 기능을 포함합니다:

### 검증 항목

1. **중복 제거**: 동일한 표층형의 항목 자동 제거
2. **형식 검증**: MeCab CSV 형식 준수 확인
3. **한글 검증**: 유효한 한글 문자 확인
4. **품사 태그 검증**: 올바른 품사 태그 사용 확인
5. **종성 일관성**: 종성 마커와 표층형 일치 확인

### 검증 비활성화

대량 생성 시 성능을 위해 검증을 비활성화할 수 있습니다:

```bash
python expand_proper_nouns.py \
    --source wikipedia \
    --no-validate \
    --no-deduplicate \
    -o output.csv
```

## 데이터 소스

### Wikipedia

- **한국어 위키피디아 API** 사용
- 카테고리별 항목 추출
- 속도 제한: 1초당 1회 요청

주요 카테고리 예시:
- `대한민국의_배우`
- `대한민국의_가수`
- `대한민국의_도시`
- `대한민국의_대학`
- `대한민국의_기업`

### 공공데이터

- 정부 기관명
- 행정구역 정보
- 확장 가능한 구조

## 기존 도구와의 연동

`/scripts` 디렉토리의 기존 도구와 연동:

### corpus_to_dict.py

```bash
# 코퍼스에서 추출한 신조어를 사전 항목으로 변환
python ../../scripts/corpus_to_dict.py \
    --input corpus_neologisms.txt \
    --output neologisms.csv

# dict-expander로 확장
python expand_proper_nouns.py \
    --input neologisms.csv \
    --type proper \
    -o expanded_neologisms.csv
```

### merge_dictionaries.py

```bash
# 여러 확장 사전 병합
python ../../scripts/merge_dictionaries.py \
    --base base_dict/ \
    --additional proper_nouns.csv compounds.csv abbreviations.csv \
    --output merged_dict/ \
    --deduplicate
```

## 커스터마이징

### 사용자 정의 패턴 추가

`expand_compounds.py`의 패턴을 확장하려면:

```python
# 패턴 파일 생성 (patterns.txt)
컴퓨터 과학
인공 지능
기계 학습
자연 언어 처리

# 실행
python expand_compounds.py \
    --dict /path/to/dict \
    --patterns patterns.txt \
    -o custom_compounds.csv
```

### 약어 규칙 추가

`expand_abbreviations.py`의 `ABBREVIATION_PATTERNS` 수정:

```python
ABBREVIATION_PATTERNS = {
    'korean': [
        lambda words: "".join(w[0] for w in words if w),
        # 사용자 정의 패턴 추가
        lambda words: "".join(w[:2] for w in words if w)[:4],
    ],
}
```

## 성능 고려사항

### 메모리 사용

- 대량 데이터 처리 시 메모리 사용량 주의
- `--limit` 옵션으로 생성 수 제한

### 생성 속도

- Wikipedia API: 속도 제한 (1초당 1회)
- 복합명사 조합: 조합 폭발 주의 (`--max-components` 제한)

### 권장사항

```bash
# 단계별 생성
python expand_proper_nouns.py --limit 1000 ...
python expand_compounds.py --max-components 2 --max-length 10 ...

# 배치 처리
for category in "배우" "가수" "운동선수"; do
    python expand_proper_nouns.py \
        --source wikipedia \
        --category "대한민국의_${category}" \
        -o "output/${category}.csv"
done
```

## 문제 해결

### Unicode 오류

```bash
# UTF-8 인코딩 확실히 하기
export PYTHONIOENCODING=utf-8
python expand_proper_nouns.py ...
```

### Wikipedia API 타임아웃

```bash
# 재시도 또는 limit 줄이기
python expand_proper_nouns.py \
    --source wikipedia \
    --limit 100 \
    ...
```

### 메모리 부족

```bash
# 청크 단위로 처리
python expand_compounds.py \
    --dict subset1/ \
    -o output1.csv

python expand_compounds.py \
    --dict subset2/ \
    -o output2.csv
```

## 라이선스

MeCab-Ko 프로젝트의 라이선스를 따릅니다 (GPL, LGPL, BSD).

## 기여

버그 리포트와 기능 제안은 GitHub Issues를 사용해주세요.

## 참고

- [MeCab-Ko 프로젝트](https://bitbucket.org/eunjeon/mecab-ko)
- [MeCab-Ko-Dic](https://bitbucket.org/eunjeon/mecab-ko-dic)
- 관련 도구: `/scripts` 디렉토리
- 신조어 수집: `/tools/neologism-collector`

## 버전

- v1.0.0 (2025-01-06): 초기 릴리스
  - 고유명사 확장
  - 복합명사 생성
  - 활용형 생성
  - 약어 확장
  - Wikipedia/공공데이터 소스
  - 품질 검증 시스템
