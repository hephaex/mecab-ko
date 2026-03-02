# 복합명사 분해 알고리즘 (Compound Noun Decomposition)

## 개요

Nori (Lucene Korean Analyzer) 호환 복합명사 분해 기능을 제공합니다. 한국어 복합명사를 자연스러운 경계에서 구성 요소로 분해하여 검색 정확도를 향상시킵니다.

## 분해 모드 (DecompoundMode)

### None
원본 복합명사를 그대로 출력합니다.

```
"형태소분석기" → ["형태소분석기/NNG"]
```

### Discard
분해된 구성 요소만 출력하고 원본은 제외합니다.

```
"형태소분석기" → ["형태소/NNG", "분석/NNG", "기/NNG"]
```

### Mixed (권장)
원본과 분해된 구성 요소를 모두 출력합니다. 검색 정확도와 재현율을 모두 높입니다.

```
"형태소분석기" → ["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"]
```

## 분해 알고리즘

### 1. 기본 조건
- 최소 3음절 이상 복합명사만 분해 대상
- 한글 음절로만 구성된 경우
- NNG (일반명사), NNP (고유명사) 품사 태그

### 2. 종성 패턴 분석

한글 음절의 종성(받침) 유무를 분석하여 자연스러운 경계를 찾습니다.

#### 패턴 1: 종성 없음 → 종성 있음
```
"형태소분석"
  형 (ㅇ) → 태 (ㅇ) → 소 (ㅇ) | 분 (ㄴ) → 석 (ㄱ)
  [형태소] + [분석]
```

#### 패턴 2: 종성 있음 → 종성 없음
```
"학교운동장"
  학 (ㄱ) → 교 (ㅇ) | 운 (ㄴ) → 동 (ㅇ) → 장 (ㅇ)
  [학교] + [운동장]
```

#### 패턴 3: 종성 연속 (2개 이상)
```
"국립국어원"
  국 (ㄱ) → 립 (ㅂ) | 국 (ㄱ) → 어 (ㅇ) | 원 (ㄴ)
  [국립] + [국어] + [원]
```

### 3. 접미사 감지

일반적인 한국어 접미사를 자동으로 감지하여 분리합니다.

| 접미사 | 의미 | 품사 태그 | 예시 |
|--------|------|-----------|------|
| 들 | 복수 | XSN | 학생들 → 학생/NNG + 들/XSN |
| 님 | 존칭 | XSN | 선생님 → 선생/NNG + 님/XSN |
| 분 | 존칭 | XSN | 어르신분 → 어르신/NNG + 분/XSN |
| 꾼 | 사람 | NNG | 장사꾼 → 장사/NNG + 꾼/NNG |

### 4. 접두사 감지

일반적인 한국어 접두사를 자동으로 감지하여 분리합니다.

| 접두사 | 의미 | 품사 태그 | 예시 |
|--------|------|-----------|------|
| 신 | 새로운 | XPN | 신도시 → 신/XPN + 도시/NNG |
| 구 | 옛 | XPN | 구시가지 → 구/XPN + 시가지/NNG |
| 총 | 계급 | XPN | 총감독 → 총/XPN + 감독/NNG |
| 부 | 부차적 | XPN | 부사장 → 부/XPN + 사장/NNG |
| 전 | 이전 | NNG | 전대통령 → 전/NNG + 대통령/NNG |
| 후 | 이후 | NNG | 후보자 → 후/NNG + 보자/NNG |

### 5. 과도한 분해 방지

- 최대 2개 분할점 (3개 부분)으로 제한
- 각 부분은 최소 1음절 이상
- 균등 분할보다 자연스러운 경계 우선

## 품사 태그 정확도

### 원본 품사 유지
분해된 각 부분은 원본 토큰의 품사 태그를 유지합니다.

```rust
Token { surface: "형태소분석", pos: "NNG", ... }
→ ["형태소/NNG", "분석/NNG"]
```

### 접미사/접두사 품사
접미사와 접두사는 적절한 품사 태그가 할당됩니다.

- XSN: 접미사 (들, 님, 분)
- XPN: 접두사 (신, 구, 총, 부)

## Offset 계산

문자 단위(character-level) offset을 정확하게 계산하여 Lucene Token과 호환됩니다.

```
"형태소분석"
형태소: offset 0..3
분석:   offset 3..5
```

바이트 offset이 아닌 문자 offset을 사용하므로 UTF-8 멀티바이트 문자도 정확히 처리됩니다.

## 사용 예제

### 기본 사용법

```rust
use mecab_ko_core::nori_compat::{DecompoundMode, NoriTokenizer};

let mut tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false)?;
let tokens = tokenizer.tokenize("형태소분석기")?;

for token in tokens {
    println!("{}: {} [{}]",
        token.surface,
        token.pos_tag,
        if token.is_decompound { "DECOMPOSED" } else { "ORIGINAL" }
    );
}
```

### NoriAnalyzer 사용

```rust
use mecab_ko_core::nori_compat::{DecompoundMode, NoriAnalyzer};

let mut analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;
let tokens = analyzer.analyze("형태소 분석기")?;
// 조사/어미가 자동으로 제거됨
```

## 테스트 케이스

### 기본 복합명사
```
형태소분석 → 형태소 + 분석
자연언어처리 → 자연 + 언어 + 처리
```

### 고유명사
```
대한민국 → 대한 + 민국
서울특별시 → 서울 + 특별시
```

### 한자어
```
국립국어원 → 국립 + 국어 + 원
중앙정부청사 → 중앙 + 정부 + 청사
```

### 접미사/접두사
```
학생들 → 학생 + 들
신도시 → 신 + 도시
```

### 혼합 종성
```
학교운동장 → 학교 + 운동장
도서관독서실 → 도서관 + 독서실
```

## 성능 고려사항

### 메모리 효율성
- 분해된 토큰은 필요할 때만 생성
- String 재사용 없이 새로운 String 할당 (immutability)

### 시간 복잡도
- O(n): n = 입력 문자열 길이
- 종성 검사는 상수 시간 (has_jongseong)
- 접미사/접두사 검사는 상수 개수 (10개 이하)

## 향후 개선 계획

### 사전 기반 분해
현재는 휴리스틱 기반이지만, 향후 사전 기반 분해로 전환 예정:
- 복합명사 사전 (Compound Dictionary)
- 통계 기반 분해 확률
- 기계학습 모델 적용

### 의미 기반 분해
형태소 분석 결과를 활용하여 의미 단위로 분해:
- 어근 추출
- 의미 태그 부여
- 동의어/유의어 확장

### 성능 최적화
- 분해 결과 캐싱
- Zero-copy 최적화
- SIMD 활용 (종성 검사)

## 참고 자료

- [Lucene Nori Tokenizer](https://lucene.apache.org/core/9_0_0/analysis/nori/org/apache/lucene/analysis/ko/KoreanTokenizer.html)
- [Elasticsearch Korean Analysis Plugin](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html)
- [MeCab-ko Documentation](https://bitbucket.org/eunjeon/mecab-ko/src/master/)
- [한글 형태소 분석기 비교](https://www.lucypark.kr/courses/2015-ba/text-mining.html)
