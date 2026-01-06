# CLI 사용법

`mecab-ko` 명령줄 도구는 터미널에서 빠르게 형태소 분석을 수행할 수 있는 인터페이스를 제공합니다.

## 기본 문법

```bash
mecab-ko [OPTIONS] [INPUT] [COMMAND]
```

## 입력 방식

### 직접 입력

```bash
mecab-ko "분석할 텍스트"
```

### 표준 입력 (stdin)

```bash
echo "분석할 텍스트" | mecab-ko
```

```bash
cat input.txt | mecab-ko
```

### 파이프라인

```bash
mecab-ko "첫 번째 문장" | grep "NNG"
```

## 옵션 상세

### `-d, --dicdir <PATH>`

사전 경로를 지정합니다.

```bash
mecab-ko -d /path/to/mecab-ko-dic "테스트"
```

기본적으로 내장 사전을 사용하며, 외부 사전을 지정하면 해당 사전을 우선 사용합니다.

### `-u, --user-dic <PATH>`

CSV 형식의 사용자 사전 파일을 로드합니다.

```bash
mecab-ko --user-dic user.csv "사용자 정의 단어"
mecab-ko -u custom.csv "테스트"
```

사용자 사전에 대한 자세한 내용은 [사용자 사전](user-dictionary.md) 장을 참조하세요.

### `-O, --output-format <FORMAT>`

출력 포맷을 지정합니다.

```bash
mecab-ko -O wakati "형태소 분석"
mecab-ko -O json "형태소 분석"
mecab-ko --output-format csv "형태소 분석"
```

사용 가능한 포맷:

| 포맷 | 설명 | 예시 출력 |
|------|------|----------|
| `default` | MeCab 기본 포맷 (기본값) | `안녕\tNNG` |
| `wakati` | 형태소만 공백 분리 | `안녕 하 세요` |
| `json` | JSON 배열 | `[{"surface":"안녕",...}]` |
| `csv` | CSV 헤더 포함 | `surface,pos,start,end,...` |
| `pos` | 표면형/품사 슬래시 구분 | `안녕/NNG` |
| `simple` | 표면형/품사 쌍 공백 분리 | `안녕/NNG 하/XSV` |
| `dump` | 디버그용 상세 정보 | `[000] surface="안녕" ...` |

### `-N, --nbest <N>`

상위 N개의 분석 결과를 출력합니다. (기본값: 1)

```bash
mecab-ko -N 3 "아버지가방에들어가신다"
```

### `--separator <SEP>`

wakati 출력 시 형태소 간 구분자를 지정합니다. (기본값: 공백)

```bash
mecab-ko -O wakati --separator "/" "형태소 분석"
# Output: 형태소/분석
```

### `--no-line`

라인별 처리를 비활성화하고 전체 텍스트를 하나로 처리합니다.

```bash
echo -e "첫째 줄\n둘째 줄" | mecab-ko --no-line
```

### `-a, --all`

디버그용 전체 분석 결과를 표시합니다.

```bash
mecab-ko -a "테스트"
```

### `-q, --quiet`

경고 메시지를 숨깁니다.

```bash
mecab-ko -q --user-dic user.csv "테스트"
```

## 서브커맨드

### `parse`

형태소 분석을 수행합니다. (기본 동작)

```bash
mecab-ko parse "분석할 텍스트"
mecab-ko parse --help
```

### `dict`

사전 정보를 표시합니다.

```bash
mecab-ko dict
mecab-ko dict /path/to/dictionary
```

출력 예시:

```
MeCab-Ko Dictionary Information
================================
Path: /path/to/dictionary
(Dictionary information)
```

### `version`

버전 정보를 표시합니다.

```bash
mecab-ko version
```

출력 예시:

```
mecab-ko 0.1.0
Rust implementation of Korean morphological analyzer

Features:
  - MeCab-compatible analysis
  - User dictionary support
  - Multiple output formats

Repository: https://github.com/hephaex/mecab-ko
```

## 출력 포맷 상세

### Default 포맷

MeCab 호환 형식입니다.

```bash
mecab-ko "안녕하세요"
```

```
안녕    NNG
하      XSV
세요    EP+EF
EOS
```

각 라인은 탭으로 구분된 `표면형 \t 품사정보` 형식이며, 분석이 끝나면 `EOS`가 출력됩니다.

### Wakati 포맷

형태소만 공백으로 분리하여 출력합니다.

```bash
mecab-ko -O wakati "형태소 분석을 수행합니다"
```

```
형태소 분석 을 수행 합니다
```

### JSON 포맷

프로그래밍에 편리한 JSON 배열 형식입니다.

```bash
mecab-ko -O json "안녕"
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

필드 설명:

| 필드 | 타입 | 설명 |
|------|------|------|
| `surface` | string | 표면형 |
| `pos` | string | 품사 태그 |
| `start` | number | 시작 바이트 위치 |
| `end` | number | 끝 바이트 위치 |
| `reading` | string? | 읽기 (있는 경우) |
| `lemma` | string? | 원형 (있는 경우) |

### CSV 포맷

스프레드시트 친화적인 CSV 형식입니다.

```bash
mecab-ko -O csv "테스트 문장"
```

```
surface,pos,start,end,reading,lemma
테스트,NNG,0,9,,
문장,NNG,10,16,,
```

### POS 포맷

각 형태소를 `표면형/품사` 형식으로 출력합니다.

```bash
mecab-ko -O pos "안녕하세요"
```

```
안녕/NNG
하/XSV
세요/EP+EF
```

### Simple 포맷

`표면형/품사` 쌍을 한 줄에 공백으로 연결합니다.

```bash
mecab-ko -O simple "안녕하세요"
```

```
안녕/NNG 하/XSV 세요/EP+EF
```

### Dump 포맷

디버깅을 위한 상세 정보를 출력합니다.

```bash
mecab-ko -O dump "안녕"
```

```
[000] surface="안녕" pos=NNG span=[0,6)
```

## 사용 예시

### 명사 추출

```bash
mecab-ko "오늘 날씨가 좋습니다" | grep "NNG\|NNP"
```

### 형태소 수 세기

```bash
mecab-ko -O wakati "긴 문장을 분석합니다" | tr ' ' '\n' | wc -l
```

### 파일 일괄 처리

```bash
for file in *.txt; do
    mecab-ko -O json < "$file" > "${file%.txt}.json"
done
```

### JSON을 jq로 처리

```bash
mecab-ko -O json "안녕하세요" | jq '.[] | .surface'
```

### 사용자 사전으로 분석 결과 비교

```bash
echo "딥러닝 기술" | mecab-ko
echo "딥러닝 기술" | mecab-ko --user-dic tech.csv
```

## 종료 코드

| 코드 | 의미 |
|------|------|
| 0 | 성공 |
| 1 | 일반 오류 |
| 2 | 명령줄 인자 오류 |

## 도움말

전체 옵션 목록:

```bash
mecab-ko --help
mecab-ko -h
```

특정 서브커맨드 도움말:

```bash
mecab-ko parse --help
mecab-ko dict --help
```
