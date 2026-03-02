# S15-06: 복합명사 분해 개선 세션 로그

**날짜**: 2026-03-03
**작업**: Sprint 15 - 사전 품질 & 정확도
**우선순위**: P2 (Medium)

## 작업 개요

Nori (Lucene Korean Analyzer) 호환 복합명사 분해 알고리즘을 개선했습니다. 종성 패턴 분석 강화, 접미사/접두사 자동 감지, 과도한 분해 방지 등 핵심 기능을 구현했습니다.

## 문제 분석

### 기존 문제점
1. **종성 패턴 분석 단순**: 단순히 종성 있는 음절 다음을 분해 지점으로 선정
2. **접미사/접두사 미감지**: "학생들", "신도시" 등 명확한 패턴을 인식하지 못함
3. **최소 음절 제약 부족**: 2음절 단어도 분해 시도하여 과도한 분해 발생
4. **품사 태그 부정확**: 접미사/접두사에 원본 품사 태그 그대로 사용

### 요구사항
1. DecompoundMode (None/Discard/Mixed) 로직 정확성
2. 자연스러운 경계 탐지 (종성 패턴)
3. 접미사/접두사 자동 인식
4. Offset 계산 정확도 (character-level)
5. Nori 호환성 유지

## 구현 내용

### 1. 종성 패턴 분석 알고리즘 개선

**위치**: `rust/crates/mecab-ko-core/src/nori_compat.rs:253-540`

#### 3가지 자연스러운 경계 패턴
```rust
// 패턴 1: 종성 없음 → 종성 있음
// "형태소분석" → "형태소" (ㅇ) + "분석" (ㄱ)
if !prev_has_jong && curr_has_jong {
    true
}

// 패턴 2: 종성 있음 → 종성 없음
// "학교운동장" → "학교" (ㄱ) + "운동장" (ㅇ)
else if prev_has_jong && !curr_has_jong {
    true
}

// 패턴 3: 종성 연속 (2개 이상)
// "국립국어원" → "국립" (ㄱㅂ) + "국어" (ㄱ) + "원" (ㄴ)
else if prev_has_jong && curr_has_jong && i >= 2 {
    has_jongseong(chars[i - 2]) == Some(true)
}
```

#### 최소 음절 제약
- **변경 전**: 2음절 이상
- **변경 후**: 3음절 이상 (과도한 분해 방지)

#### 분해 제한
- 최대 2개 분할점 (3개 부분)
- 각 부분 최소 1음절 이상

### 2. 접미사 자동 감지

**함수**: `try_extract_suffix(token, text)`

```rust
let suffixes = [
    ("들", "XSN"), // 복수 접미사
    ("님", "XSN"), // 존칭 접미사
    ("분", "XSN"), // 존칭 접미사
    ("꾼", "NNG"), // 사람 접미사
];
```

**예시**:
- "학생들" → "학생/NNG" + "들/XSN"
- "선생님" → "선생/NNG" + "님/XSN"

### 3. 접두사 자동 감지

**함수**: `try_extract_prefix(token, text)`

```rust
let prefixes = [
    ("신", "XPN"), // 새 접두사
    ("구", "XPN"), // 옛 접두사
    ("총", "XPN"), // 계급 접두사
    ("부", "XPN"), // 계급 접두사
    ("전", "NNG"), // 시간 접두사
    ("후", "NNG"), // 시간 접두사
];
```

**예시**:
- "신도시" → "신/XPN" + "도시/NNG"
- "전대통령" → "전/NNG" + "대통령/NNG"

### 4. 통합 분해 로직

**함수**: `decompound_token_enhanced(token, text)`

```rust
fn decompound_token_enhanced(token: &Token, text: &str) -> Vec<NoriToken> {
    // 1. 접미사 검사
    if let Some(tokens) = Self::try_extract_suffix(token, text) {
        return tokens;
    }

    // 2. 접두사 검사
    if let Some(tokens) = Self::try_extract_prefix(token, text) {
        return tokens;
    }

    // 3. 기본 복합명사 분해
    Self::decompound_token(token, text)
}
```

**우선순위**:
1. 접미사 (가장 명확)
2. 접두사
3. 종성 패턴 분석

## 테스트 결과

### 단위 테스트: 25개 통과

```bash
test result: ok. 25 passed; 0 failed; 0 ignored
```

**주요 테스트 케이스**:
1. `test_decompound_token_basic` - 기본 복합명사 분해
2. `test_decompound_token_short_word` - 짧은 단어 (분해 제외)
3. `test_decompound_token_non_hangul` - 비한글 (분해 제외)
4. `test_decompound_token_mixed_jongseong` - 혼합 종성 패턴
5. `test_compound_noun_patterns` - 다양한 패턴
6. `test_decompound_offset_accuracy` - offset 정확도
7. `test_decompound_min_syllable_constraint` - 최소 음절 제약
8. `test_decompound_modes_with_compound` - DecompoundMode 동작
9. `test_mixed_mode_returns_both` - Mixed 모드
10. `test_discard_mode_returns_only_parts` - Discard 모드

### Clippy: 0 경고

```bash
Checking mecab-ko-core v0.2.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.45s
```

**해결한 경고**:
- `unused_variables` - 미사용 변수 제거
- `unused_self` - static 메서드로 변경
- `inefficient_to_string` - `(*str).to_string()` 최적화
- `vec_init_then_push` - `vec![...]` 매크로 사용
- `let_and_return` - 불필요한 let 바인딩 제거

## 문서화

### 1. 알고리즘 문서

**파일**: `docs/research/algorithms/compound-noun-decomposition.md`

**내용**:
- 분해 모드 (None/Discard/Mixed) 설명
- 종성 패턴 분석 알고리즘
- 접미사/접두사 감지 규칙
- Offset 계산 방법
- 사용 예제
- 성능 고려사항
- 향후 개선 계획

### 2. 예제 개선

**파일**: `rust/crates/mecab-ko-core/examples/compound_noun_demo.rs`

**8가지 테스트 패턴**:
1. 형태소분석 - Basic compound
2. 자연언어처리 - Three-part compound
3. 대한민국 - Proper noun compound
4. 국립국어원 - Sino-Korean compound
5. 학교운동장 - Mixed jongseong
6. 학생들 - Suffix pattern
7. 신도시 - Prefix pattern
8. 형태소분석기 - Complex compound

**출력 형식**:
```
Input: 형태소분석 (Basic compound: morpheme + analysis)
Tokens: 2
  1. 형태소   [NNG] offset: 0.. 3 [DECOMPOSED]
  2. 분석     [NNG] offset: 3.. 5 [DECOMPOSED]
```

### 3. 테스트 스크립트

**파일**: `rust/test-nori.sh`

```bash
#!/bin/bash
1. Running clippy...
2. Running tests...
3. Running example...
```

## 파일 변경 요약

### 수정된 파일 (12개)
1. `rust/crates/mecab-ko-core/src/nori_compat.rs` - 알고리즘 개선
2. `rust/crates/mecab-ko-core/examples/compound_noun_demo.rs` - 예제 개선
3. `docs/research/algorithms/compound-noun-decomposition.md` - 문서 추가
4. `rust/test-nori.sh` - 테스트 스크립트 추가
5. `PLAN.md` - S15-06 완료 표시
6. `PROGRESS.md` - 세부 진행 상황 기록

### 코드 통계
- **추가**: +1121 lines
- **삭제**: -78 lines
- **순증가**: +1043 lines

## 기술적 하이라이트

### 1. Immutability 준수

```rust
// vec! 매크로 사용 (vec_init_then_push 경고 해결)
let result = vec![
    NoriToken { /* stem */ },
    NoriToken { /* suffix */ },
];
```

### 2. Static 메서드 활용

```rust
// self를 사용하지 않는 메서드는 static으로
fn try_extract_suffix(token: &Token, text: &str) -> Option<Vec<NoriToken>>
fn try_extract_prefix(token: &Token, text: &str) -> Option<Vec<NoriToken>>
fn decompound_token_enhanced(token: &Token, text: &str) -> Vec<NoriToken>
```

### 3. 효율적인 String 변환

```rust
// 변경 전: suffix.to_string() (느림)
// 변경 후: (*suffix).to_string() (빠름)
surface: (*suffix).to_string(),
```

### 4. Character Offset 정확도

```rust
fn char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset.min(text.len())].chars().count()
}
```

UTF-8 멀티바이트 문자 (한글)도 정확히 처리.

## Nori 호환성

### DecompoundMode

| Mode | 동작 | 예시 |
|------|------|------|
| None | 원본만 | ["형태소분석기"] |
| Discard | 분해만 | ["형태소", "분석", "기"] |
| Mixed | 원본+분해 | ["형태소분석기", "형태소", "분석", "기"] |

### WordType

- Known: 사전 등록 단어
- Unknown: 미등록어
- User: 사용자 사전

### Offset

- Character-level (Lucene Token 호환)
- Byte offset → Character offset 변환

## 향후 개선 계획

### 1. 사전 기반 분해
현재는 휴리스틱 기반이지만, 향후 사전 기반 분해로 전환:
- 복합명사 사전 구축
- 통계 기반 분해 확률
- 기계학습 모델 적용

### 2. 의미 기반 분해
형태소 분석 결과를 활용:
- 어근 추출
- 의미 태그 부여
- 동의어/유의어 확장

### 3. 성능 최적화
- 분해 결과 캐싱 (LRU Cache)
- Zero-copy 최적화
- SIMD 활용 (종성 검사)

## 학습 포인트

1. **종성 패턴의 중요성**: 한국어 복합명사의 자연스러운 경계는 종성 유무 전환 지점에 있음
2. **접미사/접두사 우선**: 명확한 패턴이 있는 경우 종성 분석보다 우선 적용
3. **과도한 분해 방지**: 최소 음절 제약 및 최대 분할 제한으로 품질 향상
4. **Clippy 활용**: Rust 관용구 (idioms)를 학습하고 코드 품질 개선
5. **테스트 주도**: 경계 케이스 (짧은 단어, 비한글 등)를 먼저 테스트로 작성

## 참고 자료

- [Lucene Nori Tokenizer](https://lucene.apache.org/core/9_0_0/analysis/nori/org/apache/lucene/analysis/ko/KoreanTokenizer.html)
- [Elasticsearch Nori Plugin](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html)
- [한글 자모 처리 (mecab-ko-hangul)](https://docs.rs/mecab-ko-hangul/)
- [Rust Clippy Lints](https://rust-lang.github.io/rust-clippy/rust-1.92.0/)

## 다음 작업

Sprint 15 남은 작업:
- [ ] S15-07: 성능 벤치마크 CI 통합
- [ ] S15-08: 문서 사이트 개선

---

**작업 완료**: 2026-03-03
**커밋**: 5071019b feat(nori): Improve compound noun decomposition algorithm
**테스트**: 25 passed, 0 failed, 0 ignored
**Clippy**: 0 warnings
