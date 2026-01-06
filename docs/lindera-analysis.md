# Lindera 코드베이스 분석 보고서

> **문서 버전**: 1.0
> **작성일**: 2026-01-04
> **이슈**: RST-001 Lindera 코드베이스 분석 및 fork 전략 수립

---

## 목차

1. [개요](#1-개요)
2. [아키텍처 분석](#2-아키텍처-분석)
3. [한국어 지원 현황](#3-한국어-지원-현황)
4. [코드 품질 평가](#4-코드-품질-평가)
5. [라이센스 호환성](#5-라이센스-호환성)
6. [Fork vs 신규 개발 결정](#6-fork-vs-신규-개발-결정)

---

## 1. 개요

### 1.1 Lindera 프로젝트

- **GitHub**: https://github.com/lindera-morphology/lindera
- **버전**: 1.5.1 (2026년 1월)
- **라이센스**: MIT
- **기원**: kuromoji-rs 포크
- **월간 다운로드**: ~40,000
- **의존 프로젝트**: 887개

### 1.2 지원 언어

| 언어 | Crate | 사전 |
|------|-------|------|
| 일본어 | lindera-ipadic | IPADIC |
| 일본어 | lindera-unidic | UniDic |
| 일본어 | lindera-ipadic-neologd | NEologd |
| 한국어 | lindera-ko-dic | mecab-ko-dic |
| 중국어 | lindera-cc-cedict | CC-CEDICT |

---

## 2. 아키텍처 분석

### 2.1 Crate 구조

```
lindera/
├── lindera/              # 핵심 라이브러리 (Tokenizer, Segmenter, Filters)
├── lindera-dictionary/   # 사전 코어 (Viterbi, Lattice, Trie)
├── lindera-cli/          # CLI 도구
├── lindera-ipadic/       # 일본어 IPADIC
├── lindera-unidic/       # 일본어 UniDic
├── lindera-ko-dic/       # 한국어 mecab-ko-dic
└── lindera-cc-cedict/    # 중국어 CC-CEDICT
```

### 2.2 핵심 데이터 구조

```rust
// 토크나이저 파이프라인
pub struct Tokenizer {
    segmenter: Segmenter,
    character_filters: Vec<BoxCharacterFilter>,
    token_filters: Vec<BoxTokenFilter>,
}

// Lattice 구조
pub struct Lattice {
    edges: Vec<Edge>,
    starts_at: Vec<Vec<EdgeId>>,
    ends_at: Vec<Vec<EdgeId>>,
}

// 사전 구조
pub struct Dictionary {
    pub prefix_dictionary: PrefixDictionary,  // Double Array Trie
    pub connection_cost_matrix: ConnectionCostMatrix,
    pub character_definition: CharacterDefinition,
    pub unknown_dictionary: UnknownDictionary,
}
```

### 2.3 알고리즘

| 기능 | 알고리즘/구현 |
|------|--------------|
| 사전 검색 | Double Array Trie (yada 라이브러리) |
| 형태소 분석 | Viterbi 알고리즘 |
| 사전 압축 | Deflate/Zlib/Gzip |
| 직렬화 | rkyv (zero-copy) |

### 2.4 토큰화 파이프라인

```
입력 텍스트
    ↓
[Character Filters] → 유니코드 정규화, 문자 매핑
    ↓
[Segmenter] → Lattice 구성 → Viterbi → 최적 경로
    ↓
[Token Filters] → 품사 필터링, 복합어 처리
    ↓
토큰 리스트
```

---

## 3. 한국어 지원 현황

### 3.1 lindera-ko-dic

| 항목 | 내용 |
|------|------|
| 사전 버전 | mecab-ko-dic 2.1.1-20180720 |
| 라이센스 | MIT (래퍼) + Apache-2.0 (사전) |
| CSV 형식 | 12컬럼 (세종 태그 기반) |

### 3.2 지원 기능

| 기능 | 상태 | 비고 |
|------|------|------|
| 기본 형태소 분석 | ✅ 지원 | mecab-ko-dic 품질 |
| 품사 태깅 | ✅ 지원 | 세종 태그셋 |
| 복합어 정보 | ✅ 지원 | Expression 필드 |
| 사용자 사전 | ✅ 지원 | 3컬럼/12컬럼 |
| **띄어쓰기 패널티** | ❌ 미지원 | **핵심 누락** |
| 한글 특화 미등록어 | ❌ 미지원 | 범용 처리만 |

### 3.3 핵심 문제: 띄어쓰기 패널티 미구현

mecab-ko의 `left-space-penalty-factor` 기능이 Lindera에 없음:

```
# mecab-ko의 dicrc 설정
left-space-penalty-factor = 120,6000  # 품사ID 120(조사)에 6000 페널티
```

**영향**:
```
입력: "아버지가방에들어가신다"
정상: 아버지가 + 방에 + 들어가신다 (아버지가 방에 들어가신다)
Lindera: 아버지 + 가방에 + 들어가신다 (오분석)
```

### 3.4 사전 최신성 문제

- 현재 사전: 2018년 7월 (6년+ 경과)
- 신조어, 외래어 미반영
- 업데이트 메커니즘 부재

---

## 4. 코드 품질 평가

### 4.1 종합 점수: 4.4/5

| 항목 | 점수 | 비고 |
|------|------|------|
| Idiomatic Rust | 4.5/5 | Cow, 빌더 패턴, 트레이트 활용 |
| 테스트 커버리지 | 4.0/5 | 통합 테스트 양호, 커버리지 수치 없음 |
| 문서화 | 3.5/5 | API 문서 29%, 개선 필요 |
| 의존성 관리 | 4.5/5 | 정기 업데이트, 검증된 crate |
| 프로젝트 활성도 | 4.5/5 | 월 1-2회 릴리스 |
| API 설계 | 4.5/5 | 일관성, 유연성 |
| 에러 처리 | 4.5/5 | thiserror + anyhow |
| unsafe 사용 | 5.0/5 | 핵심 로직에 없음 |

### 4.2 강점

1. **메모리 안전성**: unsafe 코드 없이 Safe Rust로 구현
2. **성능 최적화**: Cow, mmap, 버퍼 재사용
3. **확장성**: 트레이트 기반 필터 시스템
4. **성숙한 에러 처리**: 14종 에러 타입, 체이닝 지원

### 4.3 약점

1. **문서화 부족**: 공개 API 70% 미문서화
2. **한국어 특화 부재**: 띄어쓰기 패널티, 자모 처리 없음
3. **테스트 깊이**: 퍼징, 벤치마크 부족

---

## 5. 라이센스 호환성

### 5.1 라이센스 매트릭스

| 구성요소 | 라이센스 | 상업적 사용 | mecab-ko 호환 |
|----------|----------|-------------|---------------|
| Lindera 코어 | MIT | ✅ | ✅ |
| lindera-ko-dic 래퍼 | MIT | ✅ | ✅ |
| mecab-ko-dic 사전 | Apache-2.0 | ✅ | ✅ |
| 의존성 | MIT/Apache-2.0 | ✅ | ✅ |

### 5.2 결론

- **완벽 호환**: mecab-ko의 `MIT OR Apache-2.0`과 Lindera 모두 호환
- **상업적 사용**: 라이센스 고지 포함 시 가능
- **Fork 가능**: MIT 라이센스로 자유로운 수정/배포

---

## 6. Fork vs 신규 개발 결정

### 6.1 옵션 비교

| 항목 | Lindera Fork | 신규 개발 |
|------|--------------|-----------|
| 초기 개발 비용 | 낮음 | 높음 |
| 한국어 최적화 | 수정 필요 | 처음부터 설계 |
| 코드 이해 비용 | 중간 | 없음 |
| 업스트림 동기화 | 필요 | 불필요 |
| 아키텍처 자유도 | 제한적 | 완전 자유 |
| 커뮤니티 기여 | 가능 | 새로 구축 |

### 6.2 권장 결정: **하이브리드 접근**

```
┌─────────────────────────────────────────────────────────────┐
│                     권장 전략: 하이브리드                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [신규 개발]                    [Lindera 참조/차용]          │
│  ─────────                      ──────────────────          │
│  • mecab-ko-hangul (완료)       • Double Array Trie 알고리즘 │
│  • 띄어쓰기 패널티              • Viterbi 구현 패턴          │
│  • 한글 미등록어 처리           • 사전 바이너리 포맷          │
│  • dicrc 파싱                   • 필터 시스템 설계           │
│  • left-space-penalty          • 에러 처리 패턴             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### 6.3 결정 근거

**신규 개발 선택 이유**:
1. **핵심 기능 누락**: 띄어쓰기 패널티가 Lindera 아키텍처에 없음
2. **한국어 특화 필요**: 자모 처리, 조사/어미 연결 규칙
3. **mecab-ko-hangul 완료**: 이미 한글 처리 crate 구현됨
4. **학습 목적**: 형태소 분석기 내부 이해

**Lindera 참조 이유**:
1. **검증된 알고리즘**: Viterbi, Trie 구현 참고
2. **Rust 패턴**: 에러 처리, API 설계 참고
3. **사전 포맷**: 바이너리 형식 호환 가능

### 6.4 구현 로드맵

```
Phase 1: 코어 구현 (신규)
├── mecab-ko-core
│   ├── pos_tag.rs (완료)
│   ├── lattice.rs
│   ├── viterbi.rs
│   └── space_penalty.rs  ← 핵심 차별화
│
Phase 2: 사전 구현 (Lindera 참조)
├── mecab-ko-dict
│   ├── trie.rs (Lindera 참조)
│   ├── loader.rs
│   ├── matrix.rs
│   └── builder.rs
│
Phase 3: 통합
├── mecab-ko-cli
└── Python 바인딩
```

---

## 부록: Lindera 핵심 코드 참조

### A. Viterbi 알고리즘 (참조용)

```rust
// lindera-dictionary/src/viterbi.rs
pub fn calculate_path_costs(&mut self, cost_matrix: &ConnectionCostMatrix) {
    for i in 0..self.char_positions.len() {
        for edge_id in &self.ends_at[i] {
            let edge = &self.edges[*edge_id];
            let cost = cost_matrix.cost(
                edge.word_entry.left_id,
                prev_edge.word_entry.right_id
            );
            // 최소 비용 경로 선택
        }
    }
}
```

### B. 토큰 필터 트레이트 (참조용)

```rust
pub trait TokenFilter: 'static + Send + Sync + TokenFilterClone {
    fn name(&self) -> &'static str;
    fn apply(&self, tokens: &mut Vec<Token<'_>>) -> LinderaResult<()>;
}
```

### C. 에러 처리 패턴 (참조용)

```rust
pub enum LinderaErrorKind {
    Args, Algorithm, Content, Decode, Deserialize,
    Io, Parse, Serialize, Compression, NotFound,
    Build, Dictionary, Mode, FeatureDisabled
}
```

---

## 참고 자료

- [Lindera GitHub](https://github.com/lindera-morphology/lindera)
- [lindera-ko-dic](https://lib.rs/crates/lindera-ko-dic)
- [mecab-ko-dic](https://bitbucket.org/eunjeon/mecab-ko-dic)
- [Apache Lucene Nori](https://lucene.apache.org/core/)
