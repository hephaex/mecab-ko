# Nori Compatibility Layer

Lucene Nori와 호환되는 한국어 형태소 분석 인터페이스입니다.

## 개요

Apache Lucene의 한국어 분석기인 Nori는 Elasticsearch에서 널리 사용되는 형태소 분석기입니다. 이 모듈은 MeCab-Ko 기반의 형태소 분석을 Nori와 유사한 인터페이스로 제공하여 기존 Nori 사용자가 쉽게 전환할 수 있도록 합니다.

## 주요 기능

### 1. NoriTokenizer

Nori 스타일의 토크나이저로, 다음 기능을 제공합니다:

- **복합명사 분해 모드** (Decompound Mode)
  - `None`: 분해하지 않음
  - `Discard`: 원본은 버리고 분해된 형태소만 출력
  - `Mixed`: 원본과 분해된 형태소 모두 출력

- **미등록어 유니그램 출력**
  - 사전에 없는 단어를 음절 단위로 분리

### 2. NoriAnalyzer

Nori 스타일의 분석기로, 다음 기능을 제공합니다:

- **Stoptags 필터링**: 특정 품사 태그 제거
- **사용자 사전 지원** (향후 구현)
- **기본 설정**: 조사(J), 어미(E) 자동 제거

### 3. POS 태그 매핑

MeCab-Ko의 세밀한 품사 태그를 Nori의 통합 태그로 변환:

| MeCab 태그 | 설명 | Nori 태그 |
|-----------|------|----------|
| JKS, JKO, JKG, JKB, JKV, JKQ, JX, JC | 조사 | J |
| EP, EF, EC, ETN, ETM | 어미 | E |
| NNG, NNP, VV, VA 등 | 기타 | 그대로 유지 |

## 사용 예제

### 기본 토크나이저

```rust
use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};

let tokenizer = NoriTokenizer::new(DecompoundMode::None, false)?;
let tokens = tokenizer.tokenize("한국어 형태소 분석기")?;

for token in tokens {
    println!("{}: {}", token.surface, token.pos_tag);
}
```

### 복합명사 분해

```rust
use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};

// Mixed 모드: 원본 + 분해된 형태소
let tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false)?;
let tokens = tokenizer.tokenize("형태소분석기")?;

// 출력 예상:
// "형태소분석기/NNG"
// "형태소/NNG"
// "분석/NNG"
// "기/NNG"
```

### Nori Analyzer (stoptags)

```rust
use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};

// 기본 설정: 조사(J), 어미(E) 제거
let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;
let tokens = analyzer.analyze("안녕하세요")?;

// 조사와 어미가 제거된 결과만 반환
for token in tokens {
    println!("{}: {}", token.surface, token.pos_tag);
}
```

### 커스텀 Stoptags

```rust
use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};

// 조사, 어미, 마침표 제거
let stoptags = vec!["J".to_string(), "E".to_string(), "SF".to_string()];
let analyzer = NoriAnalyzer::new(
    None,
    DecompoundMode::None,
    stoptags,
    false
)?;

let tokens = analyzer.analyze("안녕하세요.")?;
// 마침표(SF)도 제거됨
```

### 동적 Stoptags 관리

```rust
let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None)?;

// 태그 추가
analyzer.add_stoptag("SN".to_string()); // 숫자 제거

// 태그 제거
analyzer.remove_stoptag("E"); // 어미 필터링 해제

// 현재 stoptags 확인
println!("Current stoptags: {:?}", analyzer.stoptags());
```

### POS 태그 변환

```rust
use mecab_ko_core::nori_compat::{mecab_to_nori_tag, nori_to_mecab_tag};

// MeCab → Nori
assert_eq!(mecab_to_nori_tag("JKS"), "J");  // 주격조사 → J
assert_eq!(mecab_to_nori_tag("EF"), "E");   // 종결어미 → E
assert_eq!(mecab_to_nori_tag("NNG"), "NNG"); // 일반명사 → NNG

// Nori → MeCab (대표 태그)
assert_eq!(nori_to_mecab_tag("J"), "JX");   // J → 보조사(대표)
assert_eq!(nori_to_mecab_tag("E"), "EF");   // E → 종결어미(대표)
```

## API 설계 철학

### Lucene Nori와의 차이점

1. **품사 태그 체계**
   - Nori: 간소화된 태그 (J, E 통합)
   - MeCab-Ko: 세밀한 세종 품사 태그
   - 호환 레이어: 양방향 변환 제공

2. **복합명사 분해**
   - Nori: 사전 기반 분해
   - MeCab-Ko: Lattice + Viterbi 기반 최적 경로
   - 호환 레이어: MeCab 결과를 Nori 스타일로 변환

3. **사용자 사전**
   - Nori: CSV 형식
   - MeCab-Ko: 바이너리 Trie
   - 호환 레이어: 향후 CSV 로더 추가 예정

### 성능 고려사항

- **Zero-cost abstraction**: 변환 오버헤드 최소화
- **Lazy evaluation**: 필요한 경우에만 복합명사 분해
- **메모리 효율**: 토큰 재사용 및 최소 할당

## 향후 계획

- [ ] 복합명사 자동 분해 구현
- [ ] CSV 형식 사용자 사전 로더
- [ ] Elasticsearch 플러그인 호환성
- [ ] 성능 최적화 (벤치마크 기반)

## 참고 자료

- [Apache Lucene Nori](https://lucene.apache.org/core/9_0_0/analysis/nori/overview-summary.html)
- [Elasticsearch Korean Analysis](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html)
- [MeCab-Ko](https://bitbucket.org/eunjeon/mecab-ko)
- [세종 품사 태그](https://ithub.korean.go.kr/user/guide/corpus/guide1.do)
