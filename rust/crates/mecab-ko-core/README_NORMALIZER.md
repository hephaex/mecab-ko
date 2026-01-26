# Foreign Word Normalization (외래어 표기 정규화)

## 개요

MeCab-Ko Core의 외래어 표기 정규화 모듈은 한국어 외래어의 다양한 표기 변이를 표준형으로 정규화합니다.

## 빠른 시작

```rust
use mecab_ko_core::normalizer::Normalizer;

// 정규화기 생성
let normalizer = Normalizer::default()?;

// 변이형을 표준형으로 정규화
assert_eq!(normalizer.normalize("코피"), "커피");
assert_eq!(normalizer.normalize("케익"), "케이크");
assert_eq!(normalizer.normalize("소프트웨아"), "소프트웨어");

// 변이형 여부 확인
assert!(normalizer.is_variant("커피", "코피"));

// 발음 유사도 계산
let similarity = normalizer.phonetic_similarity("커피", "코피");
println!("Similarity: {:.2}", similarity); // 0.75
```

## 주요 기능

### 1. 장단음 정규화
- 커피 ↔ 코피 (ㅓ ↔ ㅗ)
- 뮤직 ↔ 무직 (ㅜ ↔ ㅠ)

### 2. 자음 변이 정규화
- 베이스 ↔ 페이스 (ㅂ ↔ ㅍ)
- 데이터 ↔ 테이터 (ㄷ ↔ ㅌ)
- 가드 ↔ 카드 (ㄱ ↔ ㅋ)

### 3. 받침 변이 정규화
- 소프트웨어 ↔ 소프트웨아 (받침 추가/제거)

### 4. 발음 유사성 기반 Fuzzy Matching
- Levenshtein distance 기반
- 자모 단위 분해를 통한 정확한 측정

## Tokenizer 통합

```rust
use mecab_ko_core::Tokenizer;

let mut tokenizer = Tokenizer::new()?;

// 정규화 활성화
tokenizer.set_normalization(true, None)?;

// 정규화 적용 분석
let tokens = tokenizer.tokenize_with_normalization("코피를 마셨다");
for token in tokens {
    if let Some(normalized) = &token.normalized {
        println!("{} -> {}", token.surface, normalized);
    }
}
```

## 예제 실행

```bash
cargo run --example normalizer_example
```

## 데이터 파일

### 내장 데이터
- 45+ 일반 외래어 변이형
- IT 용어 (쿠버네티스, 알고리즘, 데이터베이스 등)
- 음식/음료 용어
- 전자제품 용어

### 외부 CSV 지원
```csv
standard,variant,category,confidence,notes
커피,코피,beverage,0.95,장단음 변이
케이크,케익,food,0.95,이중모음 변이
```

## 성능

- 직접 매핑: O(1)
- 규칙 기반: O(n)
- 발음 유사도: O(n×m)
- 멀티스레드 안전 (Arc 사용)

## 테스트

```bash
cargo test --lib normalizer
```

전체 테스트 통과:
- ✓ 정규화기 생성
- ✓ 내장 변이형 정규화
- ✓ 변이형 조회
- ✓ 변이형 여부 확인
- ✓ 발음 유사도
- ✓ IT 용어
- ✓ 규칙 기반 변이형 생성

## 문서

자세한 문서는 [docs/NORMALIZER.md](../../../docs/NORMALIZER.md)를 참조하세요.

## 라이선스

Apache-2.0 / MIT
