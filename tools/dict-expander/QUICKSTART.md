# Quick Start Guide

MeCab-Ko Dictionary Expander를 5분 안에 시작하는 가이드입니다.

## 1. 설치 확인

```bash
# Python 버전 확인 (3.10 이상 필요)
python3 --version

# 디렉토리 이동
cd /home/mare/mecab-ko/tools/dict-expander
```

## 2. 기본 테스트

```bash
# 모든 모듈이 정상 작동하는지 확인
python3 test_basic.py
```

**예상 출력**:
```
✓ PASS: Imports
✓ PASS: MeCab Format
✓ PASS: Korean Utils
✓ PASS: Validators
```

## 3. 첫 번째 사전 생성

### 예제 1: 고유명사 추출

```bash
# 샘플 인명 파일 생성
cat > sample_names.txt << EOF
김철수
이영희
박민수
최지혜
정우성
EOF

# 사전 항목 생성
python3 expand_proper_nouns.py \
    --input sample_names.txt \
    --type person \
    -o my_first_dict.csv

# 결과 확인
cat my_first_dict.csv
```

**출력 예시**:
```csv
김철수,0,0,0,NNP,인명,F,김철수,*,*,*,*
이영희,0,0,0,NNP,인명,F,이영희,*,*,*,*
박민수,0,0,0,NNP,인명,F,박민수,*,*,*,*
최지혜,0,0,0,NNP,인명,F,최지혜,*,*,*,*
정우성,0,0,0,NNP,인명,T,정우성,*,*,*,*
```

### 예제 2: 복합명사 생성

```bash
# 샘플 명사 파일 생성
cat > base_nouns.txt << EOF
컴퓨터
과학
데이터
분석
EOF

# 복합명사 생성
python3 expand_compounds.py \
    --input base_nouns.txt \
    --suffixes \
    -o compounds.csv

# 결과 확인
cat compounds.csv
```

### 예제 3: 동사 활용형 생성

```bash
# 샘플 동사 파일 생성
cat > verbs.txt << EOF
하다
가다
먹다
EOF

# 활용형 생성
python3 expand_conjugations.py \
    --input verbs.txt \
    --patterns common \
    -o conjugations.csv

# 결과 확인
head conjugations.csv
```

## 4. 완전한 워크플로우 실행

```bash
# 예제 워크플로우 실행 (모든 기능 데모)
./example_workflow.sh

# 결과 확인
ls -lh output/
```

## 5. Wikipedia 데이터 사용 (선택)

```bash
# Wikipedia에서 실제 데이터 가져오기
python3 expand_proper_nouns.py \
    --source wikipedia \
    --category "대한민국의_배우" \
    --type person \
    --limit 10 \
    -o wikipedia_actors.csv

# 결과 확인
cat wikipedia_actors.csv
```

## 6. 도움말 보기

각 도구의 자세한 옵션 확인:

```bash
# 고유명사 확장 도구
python3 expand_proper_nouns.py --help

# 복합명사 생성 도구
python3 expand_compounds.py --help

# 활용형 생성 도구
python3 expand_conjugations.py --help

# 약어 확장 도구
python3 expand_abbreviations.py --help
```

## 7. 기존 사전과 통합

생성한 사전을 MeCab-Ko 사전과 통합:

```bash
# scripts 디렉토리로 이동
cd ../../scripts

# 사전 병합
python3 merge_dictionaries.py \
    --base /path/to/mecab-ko-dic/seed \
    --additional ../tools/dict-expander/my_first_dict.csv \
    --output /path/to/output/merged_dict \
    --deduplicate
```

## 일반적인 사용 패턴

### Pattern 1: 텍스트 파일에서 사전 생성

```bash
# 1. 단어 목록 준비 (한 줄에 하나씩)
echo "단어1
단어2
단어3" > words.txt

# 2. 사전 생성
python3 expand_proper_nouns.py --input words.txt --type proper -o output.csv
```

### Pattern 2: 여러 소스 결합

```bash
# 여러 소스에서 생성
python3 expand_proper_nouns.py --input names.txt --type person -o p1.csv
python3 expand_proper_nouns.py --input places.txt --type place -o p2.csv

# 결합
cat p1.csv p2.csv > combined.csv
```

### Pattern 3: 파이프라인 구축

```bash
# 1. 고유명사 생성
python3 expand_proper_nouns.py --input names.txt -o step1.csv

# 2. 복합명사 생성
python3 expand_compounds.py --input nouns.txt --suffixes -o step2.csv

# 3. 병합
cat step1.csv step2.csv > final.csv
```

## 문제 해결

### 한글이 깨져 보이는 경우

```bash
export PYTHONIOENCODING=utf-8
python3 expand_proper_nouns.py ...
```

### 테스트가 실패하는 경우

```bash
# Python 버전 확인
python3 --version  # 3.10 이상이어야 함

# 모듈 경로 확인
python3 -c "import sys; print(sys.path)"
```

### Wikipedia API 타임아웃

```bash
# limit을 줄여서 재시도
python3 expand_proper_nouns.py \
    --source wikipedia \
    --limit 10 \
    ...
```

## 다음 단계

1. [README.md](README.md) - 전체 문서 읽기
2. [IMPLEMENTATION.md](IMPLEMENTATION.md) - 구현 상세 확인
3. [example_workflow.sh](example_workflow.sh) - 전체 워크플로우 분석

## 빠른 참조

### 주요 옵션

| 옵션 | 설명 | 예시 |
|------|------|------|
| `--input` | 입력 파일 | `--input words.txt` |
| `--output, -o` | 출력 파일 | `-o output.csv` |
| `--type` | 항목 유형 | `--type person` |
| `--source` | 데이터 소스 | `--source wikipedia` |
| `--limit` | 최대 항목 수 | `--limit 100` |
| `--no-validate` | 검증 비활성화 | `--no-validate` |

### 지원하는 유형

- **고유명사**: `person`, `place`, `organization`, `proper`
- **복합명사**: 자동 조합, 접미사, 패턴
- **활용형**: `common`, `present_informal`, `past`, `future`, `honorific`, `connecting`
- **약어**: `korean`, `english`, `both`

## 완료!

이제 MeCab-Ko Dictionary Expander를 사용할 준비가 되었습니다!

더 많은 정보는 [README.md](README.md)를 참조하세요.
