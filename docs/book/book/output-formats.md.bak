# 출력 포맷

MeCab-Ko는 다양한 출력 포맷을 지원합니다. 용도에 따라 적절한 포맷을 선택하세요.

## 포맷 비교

| 포맷 | 용도 | 파싱 용이성 | 가독성 |
|------|------|-------------|--------|
| default | MeCab 호환 | 중간 | 좋음 |
| wakati | 분리된 텍스트 | 쉬움 | 좋음 |
| json | 프로그래밍 | 매우 쉬움 | 보통 |
| csv | 스프레드시트 | 쉬움 | 보통 |
| pos | 간단한 태깅 | 쉬움 | 좋음 |
| simple | 한 줄 요약 | 쉬움 | 좋음 |
| dump | 디버깅 | 어려움 | 상세 |

## Default 포맷

MeCab 원본과 호환되는 기본 포맷입니다.

### 사용법

```bash
mecab-ko "안녕하세요"
mecab-ko -O default "안녕하세요"
```

### 출력 형식

```
표면형\t품사정보
...
EOS
```

### 예시

```bash
$ mecab-ko "오늘 날씨가 좋습니다"
```

```
오늘    NNG
날씨    NNG
가      JKS
좋      VA
습니다  EF
EOS
```

### 파싱

```python
# Python
for line in output.split('\n'):
    if line == 'EOS':
        break
    surface, pos = line.split('\t')
```

```rust
// Rust
for line in output.lines() {
    if line == "EOS" {
        break;
    }
    let parts: Vec<&str> = line.split('\t').collect();
    let surface = parts[0];
    let pos = parts[1];
}
```

## Wakati 포맷

형태소를 공백으로 분리하여 출력합니다. 텍스트 전처리에 유용합니다.

### 사용법

```bash
mecab-ko -O wakati "안녕하세요"
```

### 출력 형식

```
형태소1 형태소2 형태소3 ...
```

### 예시

```bash
$ mecab-ko -O wakati "오늘 날씨가 좋습니다"
```

```
오늘 날씨 가 좋 습니다
```

### 커스텀 구분자

```bash
# Slash separator
mecab-ko -O wakati --separator "/" "테스트"
# Output: 테/스/트

# Newline separator
mecab-ko -O wakati --separator $'\n' "테스트"
```

### 활용 예시

```bash
# Word count
mecab-ko -O wakati "긴 문장" | tr ' ' '\n' | wc -l

# Unique morphemes
mecab-ko -O wakati "문장" | tr ' ' '\n' | sort | uniq

# TF-IDF input preparation
cat document.txt | mecab-ko -O wakati > tokenized.txt
```

## JSON 포맷

프로그래밍에 가장 편리한 구조화된 형식입니다.

### 사용법

```bash
mecab-ko -O json "안녕하세요"
```

### 출력 형식

```json
[
  {
    "surface": "표면형",
    "pos": "품사",
    "start": 시작바이트,
    "end": 끝바이트,
    "reading": "읽기",
    "lemma": "원형"
  },
  ...
]
```

### 예시

```bash
$ mecab-ko -O json "안녕"
```

```json
[
  {
    "surface": "안녕",
    "pos": "NNG",
    "start": 0,
    "end": 6
  }
]
```

### 필드 설명

| 필드 | 타입 | 필수 | 설명 |
|------|------|------|------|
| `surface` | string | O | 표면형 (원문 그대로) |
| `pos` | string | O | 품사 태그 |
| `start` | number | O | 시작 바이트 위치 |
| `end` | number | O | 끝 바이트 위치 (exclusive) |
| `reading` | string | X | 읽기/발음 |
| `lemma` | string | X | 원형/기본형 |

### 활용 예시

```bash
# Extract surfaces with jq
mecab-ko -O json "안녕하세요" | jq '.[].surface'

# Filter nouns
mecab-ko -O json "오늘 날씨" | jq '[.[] | select(.pos | startswith("NN"))]'

# Count by POS
mecab-ko -O json "문장" | jq 'group_by(.pos) | map({pos: .[0].pos, count: length})'
```

```python
# Python
import json
import subprocess

result = subprocess.run(
    ["mecab-ko", "-O", "json", "안녕하세요"],
    capture_output=True, text=True
)
tokens = json.loads(result.stdout)

for token in tokens:
    print(f"{token['surface']}: {token['pos']}")
```

## CSV 포맷

스프레드시트 애플리케이션에서 열기 좋은 형식입니다.

### 사용법

```bash
mecab-ko -O csv "안녕하세요"
```

### 출력 형식

```csv
surface,pos,start,end,reading,lemma
표면형1,품사1,시작1,끝1,읽기1,원형1
...
```

### 예시

```bash
$ mecab-ko -O csv "오늘 날씨"
```

```csv
surface,pos,start,end,reading,lemma
오늘,NNG,0,6,,
날씨,NNG,7,13,,
```

### 활용 예시

```bash
# Save to file
mecab-ko -O csv "문장" > output.csv

# Open with spreadsheet application
libreoffice --calc output.csv

# Extract column
mecab-ko -O csv "문장" | cut -d',' -f1,2
```

## POS 포맷

표면형과 품사를 슬래시로 연결하여 출력합니다.

### 사용법

```bash
mecab-ko -O pos "안녕하세요"
```

### 출력 형식

```
표면형1/품사1
표면형2/품사2
...
```

### 예시

```bash
$ mecab-ko -O pos "오늘 날씨가 좋습니다"
```

```
오늘/NNG
날씨/NNG
가/JKS
좋/VA
습니다/EF
```

### 활용 예시

```bash
# Filter specific POS
mecab-ko -O pos "문장" | grep "/NNG"

# Count POS tags
mecab-ko -O pos "문장" | cut -d'/' -f2 | sort | uniq -c
```

## Simple 포맷

한 줄에 모든 분석 결과를 출력합니다.

### 사용법

```bash
mecab-ko -O simple "안녕하세요"
```

### 출력 형식

```
표면형1/품사1 표면형2/품사2 ...
```

### 예시

```bash
$ mecab-ko -O simple "오늘 날씨가 좋습니다"
```

```
오늘/NNG 날씨/NNG 가/JKS 좋/VA 습니다/EF
```

### 활용 예시

간결한 로그 출력이나 비교에 유용:

```bash
# Compare two sentences
echo "문장1: $(mecab-ko -O simple '첫 번째 문장')"
echo "문장2: $(mecab-ko -O simple '두 번째 문장')"
```

## Dump 포맷

디버깅을 위한 상세 정보를 출력합니다.

### 사용법

```bash
mecab-ko -O dump "안녕하세요"
```

### 출력 형식

```
[인덱스] surface="표면형" pos=품사 span=[시작,끝)
```

### 예시

```bash
$ mecab-ko -O dump "안녕"
```

```
[000] surface="안녕" pos=NNG span=[0,6)
```

### 활용

- 바이트 위치 확인
- 토큰 인덱스 추적
- 분석 결과 디버깅

## 포맷 선택 가이드

### 상황별 추천

| 상황 | 추천 포맷 |
|------|----------|
| 기존 MeCab 스크립트와 호환 | default |
| 텍스트 전처리/토큰화 | wakati |
| 프로그래밍에서 파싱 | json |
| 엑셀/스프레드시트 분석 | csv |
| 간단한 태깅 확인 | pos, simple |
| 문제 디버깅 | dump |

### 성능 고려

출력 포맷에 따른 성능 차이는 미미합니다. 용도에 맞는 포맷을 선택하세요.

## 커스텀 출력

프로그래밍 방식으로 커스텀 출력을 만들 수 있습니다:

```rust
use mecab_ko::Tokenizer;

let tokenizer = Tokenizer::new()?;
let tokens = tokenizer.tokenize("안녕하세요");

// Custom format: SURFACE[POS]
for token in tokens {
    print!("{}[{}] ", token.surface, token.pos);
}
println!();
// Output: 안녕[NNG] 하[XSV] 세요[EP+EF]
```

```bash
# Using shell
mecab-ko -O json "안녕하세요" | jq -r '.[] | "\(.surface)[\(.pos)]"' | tr '\n' ' '
```
