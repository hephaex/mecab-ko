# Foreign Word Normalization Module (외래어 표기 정규화 모듈)

## 개요

`mecab-ko-core`의 외래어 표기 정규화 모듈은 국립국어원 외래어 표기법을 기반으로 한국어 외래어의 다양한 표기 변이형을 표준형으로 정규화합니다.

## 주요 기능

### 1. 장단음 정규화
- **ㅓ ↔ ㅗ 변이**: 커피 ↔ 코피
- **ㅜ ↔ ㅠ 변이**: 뮤직 ↔ 무직

### 2. 자음 변이 정규화
- **ㅂ ↔ ㅍ**: 베이스 ↔ 페이스
- **ㄷ ↔ ㅌ**: 데이터 ↔ 테이터
- **ㄱ ↔ ㅋ**: 가드 ↔ 카드
- **ㅈ ↔ ㅊ**: 자켓 ↔ 차켓
- **ㅅ ↔ ㅆ**: 샤워 ↔ 샤워

### 3. 받침 변이 정규화
- **받침 추가/제거**: 소프트웨어 ↔ 소프트웨아
- **ㄹ 받침 변이**: 서버 ↔ 서벌
- **ㅁ ↔ ㅂ**: 컴퍼니 ↔ 컴퍼니

### 4. 모음 변이 정규화
- **이중모음 단순화**: 케이크 ↔ 케크, 스테이크 ↔ 스테크
- **ㅣ ↔ ㅡ**: 케이크 ↔ 케익

### 5. 발음 유사성 기반 Fuzzy Matching
- Levenshtein distance 기반 문자열 유사도 계산
- 자모 단위 분해를 통한 정확한 발음 유사도 측정

## 사용법

### 기본 사용

```rust
use mecab_ko_core::normalizer::Normalizer;

// 1. 정규화기 생성
let normalizer = Normalizer::default()?;

// 2. 표준형으로 정규화
let normalized = normalizer.normalize("코피");
assert_eq!(normalized, "커피");

// 3. 변이형 목록 조회
let variants = normalizer.get_variants("커피");
assert!(variants.contains(&"코피".to_string()));

// 4. 변이형 여부 확인
assert!(normalizer.is_variant("커피", "코피"));

// 5. 발음 유사도 계산
let similarity = normalizer.phonetic_similarity("커피", "코피");
assert!(similarity > 0.6);
```

### 커스텀 설정

```rust
use mecab_ko_core::normalizer::{NormalizationConfig, Normalizer};

let mut config = NormalizationConfig::default();
config.vowel_length = true;          // 장단음 정규화
config.consonant_variation = true;   // 자음 변이 정규화
config.jongseong_variation = true;   // 받침 변이 정규화
config.vowel_variation = true;       // 모음 변이 정규화
config.phonetic_similarity = true;   // 발음 유사성 기반 정규화
config.min_confidence = 0.7;         // 최소 신뢰도 임계값

let normalizer = Normalizer::new(config)?;
```

### Tokenizer와 통합

```rust
use mecab_ko_core::{Tokenizer, normalizer::NormalizationConfig};

let mut tokenizer = Tokenizer::new()?;

// 정규화 활성화
tokenizer.set_normalization(true, Some(NormalizationConfig::default()))?;

// 정규화 적용 분석
let tokens = tokenizer.tokenize_with_normalization("코피를 마셨다");

for token in tokens {
    println!("{} -> {:?}", token.surface, token.normalized);
}

// 변이형 확장 검색
let (standard, variants) = tokenizer.get_word_variants("코피");
println!("Standard: {}, Variants: {:?}", standard, variants);
```

### 외부 데이터 파일 사용

```rust
use mecab_ko_core::normalizer::{NormalizationConfig, Normalizer};
use std::path::Path;

let config = NormalizationConfig::default();
let variant_csv = Path::new("data/normalization/variant_map.csv");

let normalizer = Normalizer::with_data_file(config, variant_csv)?;
```

## 데이터 파일 형식

### variant_map.csv

```csv
standard,variant,category,confidence,notes
커피,코피,beverage,0.95,장단음 변이 (어↔오)
케이크,케익,food,0.95,이중모음 변이 (이)
쿠버네티스,쿠베르네테스,it,0.85,모음/자음 복합 변이
소프트웨어,소프트웨아,it,0.90,받침 제거 (ㄹ)
```

### foreign_word_rules.json

```json
{
  "version": "1.0.0",
  "description": "외래어 표기 정규화 규칙",
  "rules": [
    {
      "rule_type": "VowelLength",
      "from": "오",
      "to": "어",
      "confidence": 0.9,
      "examples": ["코피 → 커피"]
    }
  ]
}
```

## API 레퍼런스

### `Normalizer`

#### `new(config: NormalizationConfig) -> Result<Self>`
설정으로 정규화기 생성

#### `default() -> Result<Self>`
기본 설정으로 정규화기 생성

#### `with_data_file(config: NormalizationConfig, csv_path: &Path) -> Result<Self>`
외부 CSV 파일로 정규화기 생성

#### `normalize(&self, text: &str) -> String`
텍스트를 표준형으로 정규화

#### `get_variants(&self, standard: &str) -> Vec<String>`
표준형의 모든 변이형 조회

#### `is_variant(&self, word1: &str, word2: &str) -> bool`
두 단어가 변이형 관계인지 확인

#### `phonetic_similarity(&self, word1: &str, word2: &str) -> f32`
발음 유사도 계산 (0.0 ~ 1.0)

### `NormalizationConfig`

```rust
pub struct NormalizationConfig {
    pub vowel_length: bool,          // 장단음 정규화
    pub consonant_variation: bool,   // 자음 변이 정규화
    pub jongseong_variation: bool,   // 받침 변이 정규화
    pub vowel_variation: bool,       // 모음 변이 정규화
    pub phonetic_similarity: bool,   // 발음 유사성 기반 정규화
    pub min_confidence: f32,         // 최소 신뢰도 (0.0 ~ 1.0)
}
```

### `RuleType`

```rust
pub enum RuleType {
    VowelLength,           // 장단음 변이
    ConsonantVariation,    // 자음 변이
    JongseongVariation,    // 받침 변이
    VowelVariation,        // 모음 변이
    PhoneticSimilarity,    // 발음 유사성
}
```

## 성능 특성

- **정규화 속도**: O(1) for direct mapping, O(n) for rule-based
- **변이형 생성**: O(n×m) where n=word length, m=number of rules
- **발음 유사도**: O(n×m) Levenshtein distance
- **메모리 사용**: Arc로 공유되는 데이터 구조 (멀티스레드 안전)

## 테스트

```bash
# 정규화 테스트 실행
cargo test --package mecab-ko-core --lib normalizer

# 예제 실행
cargo run --package mecab-ko-core --example normalizer_example
```

## 내장 변이형 데이터

모듈은 다음 카테고리의 내장 변이형 데이터를 포함합니다:

- **음료**: 커피, 초콜릿, 코코아
- **음식**: 케이크, 스테이크, 쿠키
- **IT 용어**: 컴퓨터, 서버, 쿠버네티스, 알고리즘, 데이터베이스
- **전자제품**: 카메라, 비디오, 라디오, 텔레비전
- **교통**: 택시, 버스, 트럭

## 확장 가능성

### 사용자 정의 변이형 추가

CSV 파일에 추가하거나 `Normalizer`를 확장하여 커스텀 변이형을 추가할 수 있습니다.

### 언어별 규칙 추가

`RuleType`을 확장하여 새로운 정규화 규칙을 추가할 수 있습니다.

## 제한사항

- 현재는 한국어 외래어만 지원
- 복합어 분리는 지원하지 않음 (예: "소프트웨어개발" → 분석 안 됨)
- 문맥 정보를 고려하지 않음 (예: "코피"가 "커피" vs "코피(피)" 구분 안 됨)

## 참고 자료

- [국립국어원 외래어 표기법](https://www.korean.go.kr/front/page/pageView.do?page_id=P000105&mn_id=97)
- [한국어 형태소 분석기 Kiwi](https://github.com/bab2min/Kiwi)
- [MeCab 원본](https://taku910.github.io/mecab/)

## 라이선스

Apache License 2.0 / MIT License

## 기여

이슈와 PR은 언제나 환영합니다!
