# MeCab-Ko 사용자 사전

## 개요

이 디렉토리는 MeCab-Ko의 사용자 정의 사전 파일을 포함합니다.

## 파일 목록

| 파일 | 설명 | 항목 수 |
|------|------|---------|
| neologisms.csv | 2018-2024 신조어 | 123개 |

## CSV 형식

```
표면형,좌ID,우ID,비용,품사,분류,종류,활용형,조합읽기,원형,읽기,타입
```

### 필드 설명

| 필드 | 설명 | 예시 |
|------|------|------|
| 표면형 | 실제 텍스트 | 챗GPT |
| 좌ID | 좌문맥 ID (0=자동) | 0 |
| 우ID | 우문맥 ID (0=자동) | 0 |
| 비용 | 우선순위 (낮을수록 높음) | 0 |
| 품사 | 세종 품사 태그 | NNP |
| 분류~타입 | 추가 속성 | * |

### 주요 품사 태그

| 태그 | 설명 | 예시 |
|------|------|------|
| NNP | 고유명사 | 챗GPT, BTS |
| NNG | 일반명사 | 메타버스, 워라밸 |
| VV | 동사 | - |
| VA | 형용사 | - |
| MAG | 일반부사 | - |
| IC | 감탄사 | ㅋㅋㅋ |

## 사용법

### Rust API

```rust
use mecab_ko::UserDictionary;

let mut user_dict = UserDictionary::new();
user_dict.load_csv("data/user-dict/neologisms.csv")?;

let tokenizer = Tokenizer::builder()
    .user_dictionary(user_dict)
    .build()?;
```

### CLI

```bash
mecab --user-dict data/user-dict/neologisms.csv "챗GPT가 대세입니다"
```

## 기여 방법

1. neologisms.csv에 새 항목 추가
2. 형식 준수 확인
3. Pull Request 제출

### 품질 기준

- 실제 사용되는 단어
- 올바른 품사 태그
- 일관된 읽기 정보
- 중복 없음

## 라이선스

CC-BY-SA 4.0
