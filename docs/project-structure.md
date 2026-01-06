# MeCab-Ko Rust 프로젝트 구조

> **문서 버전**: 1.0
> **작성일**: 2026-01-04
> **이슈**: RST-002 프로젝트 구조 및 Cargo workspace 설계

---

## 목차

1. [개요](#1-개요)
2. [Crate 구조](#2-crate-구조)
3. [의존성 그래프](#3-의존성-그래프)
4. [Crate 상세](#4-crate-상세)
5. [빌드 및 테스트](#5-빌드-및-테스트)
6. [개발 로드맵](#6-개발-로드맵)

---

## 1. 개요

### 1.1 설계 원칙

| 원칙 | 설명 |
|------|------|
| **모듈화** | 기능별 독립 crate 분리 |
| **최소 의존성** | 각 crate는 필요한 의존성만 포함 |
| **안전성** | `unsafe` 코드 금지, `unwrap()`/`expect()` 라이브러리에서 금지 |
| **문서화** | 모든 public API에 rustdoc 필수 |
| **테스트** | 유닛 테스트 + 통합 테스트 |

### 1.2 하이브리드 접근법

RST-001 분석 결과에 따라 **하이브리드 접근법** 채택:

```
┌─────────────────────────────────────────────────────────────┐
│                     신규 개발                                │
├─────────────────────────────────────────────────────────────┤
│  • mecab-ko-hangul (완료) - 한글 자모 처리                   │
│  • mecab-ko-core - 띄어쓰기 패널티, Viterbi, Lattice         │
│  • mecab-ko-dict - 사전 로딩, 연접 비용                       │
│  • mecab-ko-dict-builder - CSV → 바이너리 변환               │
├─────────────────────────────────────────────────────────────┤
│                     Lindera 참조                             │
├─────────────────────────────────────────────────────────────┤
│  • Double Array Trie 알고리즘 (yada 라이브러리)               │
│  • Viterbi 구현 패턴                                         │
│  • 에러 처리 패턴 (thiserror)                                │
│  • 필터 시스템 설계                                           │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Crate 구조

### 2.1 디렉토리 레이아웃

```
rust/
├── Cargo.toml                    # Workspace 정의
├── README.md
└── crates/
    ├── mecab-ko/                 # 📦 Facade crate (통합 인터페이스)
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    │
    ├── mecab-ko-core/            # 📦 핵심 엔진
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       └── pos_tag.rs        # 품사 태그 정의
    │
    ├── mecab-ko-dict/            # 📦 사전 관리
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    │
    ├── mecab-ko-dict-builder/    # 📦 사전 빌더
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       └── main.rs           # CLI 바이너리
    │
    ├── mecab-ko-hangul/          # 📦 한글 유틸리티 (완료)
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs
    │
    └── mecab-ko-cli/             # 📦 CLI 도구
        ├── Cargo.toml
        └── src/
            └── main.rs
```

### 2.2 Crate 요약

| Crate | 타입 | 상태 | 설명 |
|-------|------|------|------|
| `mecab-ko` | lib | 스텁 | Facade - 모든 기능 재export |
| `mecab-ko-core` | lib | 스텁 | Lattice, Viterbi, Tokenizer |
| `mecab-ko-dict` | lib | 스텁 | 사전 로딩, 검색, 연접 비용 |
| `mecab-ko-dict-builder` | lib+bin | 스텁 | CSV → 바이너리 변환 |
| `mecab-ko-hangul` | lib | **완료** | 한글 자모 처리 |
| `mecab-ko-cli` | bin | 스텁 | 명령줄 도구 |

---

## 3. 의존성 그래프

```
                    ┌──────────────────┐
                    │   mecab-ko-cli   │
                    │   (바이너리)      │
                    └────────┬─────────┘
                             │
                    ┌────────▼─────────┐
                    │     mecab-ko     │
                    │   (facade lib)   │
                    └────────┬─────────┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
         ▼                   ▼                   ▼
┌────────────────┐  ┌────────────────┐  ┌────────────────┐
│ mecab-ko-core  │  │ mecab-ko-dict  │  │mecab-ko-hangul │
│  (핵심 엔진)    │  │   (사전)        │  │  (한글 처리)    │
└───────┬────────┘  └───────┬────────┘  └────────────────┘
        │                   │                   ▲
        │                   │                   │
        └───────────────────┴───────────────────┘
                            │
              ┌─────────────┴─────────────┐
              │  mecab-ko-dict-builder    │
              │    (사전 빌드 도구)         │
              └───────────────────────────┘
```

---

## 4. Crate 상세

### 4.1 mecab-ko-hangul (완료)

한글 자모 처리 유틸리티.

```rust
// 주요 기능
pub fn decompose(c: char) -> Option<(char, char, Option<char>)>;
pub fn compose(cho: char, jung: char, jong: Option<char>) -> Option<char>;
pub fn is_hangul_syllable(c: char) -> bool;
pub fn has_jongseong(c: char) -> Option<bool>;
pub fn classify_char(c: char) -> CharType;
```

**의존성**: 없음 (zero dependencies)

### 4.2 mecab-ko-core

핵심 형태소 분석 엔진.

```rust
// 주요 타입
pub struct Tokenizer { ... }
pub struct Token { ... }
pub struct Lattice { ... }
pub enum PosTag { ... }  // 41개 품사 태그

// 주요 기능
impl Tokenizer {
    pub fn new() -> Result<Self>;
    pub fn tokenize(&self, text: &str) -> Vec<Token>;
    pub fn wakati(&self, text: &str) -> Vec<String>;
    pub fn nouns(&self, text: &str) -> Vec<String>;
}
```

**핵심 구현 예정**:
- `space_penalty.rs` - 띄어쓰기 패널티 (left-space-penalty-factor)
- `viterbi.rs` - Viterbi 알고리즘
- `lattice.rs` - Lattice 구조체

**의존성**: mecab-ko-hangul, mecab-ko-dict, thiserror, serde

### 4.3 mecab-ko-dict

사전 관리 라이브러리.

```rust
// 주요 타입
pub struct Entry { ... }
pub trait Dictionary {
    fn lookup(&self, surface: &str) -> Vec<Entry>;
    fn get_connection_cost(&self, left_id: u16, right_id: u16) -> i16;
}

// 구현체
pub struct MmapDictionary { ... }  // Memory-mapped 사전
```

**핵심 구현 예정**:
- Double Array Trie (yada 라이브러리 활용)
- 연접 비용 매트릭스
- 미등록어 처리 (char.def, unk.def)

**의존성**: mecab-ko-hangul, fst, yada, memmap2, zstd, byteorder

### 4.4 mecab-ko-dict-builder

사전 빌드 도구.

```rust
// 주요 타입
pub struct DictionaryBuilder { ... }
pub struct BuildConfig { ... }
pub struct CsvEntry { ... }  // 12컬럼 CSV

// CLI 사용법
// mecab-ko-dict-builder --input ./mecab-ko-dic --output dict.bin
```

**핵심 구현 예정**:
- CSV 파싱 (33개 파일)
- Double Array Trie 빌드
- matrix.def 변환
- char.def, unk.def 변환
- Zstd 압축

**의존성**: mecab-ko-hangul, mecab-ko-dict, csv, yada, rkyv, zstd, clap, indicatif

### 4.5 mecab-ko

Facade crate - 사용자 친화적 인터페이스.

```rust
// 모든 기능을 하나로 통합
pub use mecab_ko_core::{Tokenizer, Token, PosTag};
pub use mecab_ko_hangul::{decompose, compose, is_hangul};
pub use mecab_ko_dict::{Dictionary, Entry};

// 간편 사용
use mecab_ko::Tokenizer;
let tok = Tokenizer::new()?;
let tokens = tok.tokenize("안녕하세요");
```

**의존성**: mecab-ko-core, mecab-ko-dict, mecab-ko-hangul

### 4.6 mecab-ko-cli

명령줄 도구.

```bash
# 기본 사용법
mecab-ko "안녕하세요"

# 분리만 (wakati)
mecab-ko -O wakati "안녕하세요"

# JSON 출력
mecab-ko -O json "안녕하세요"

# 사전 지정
mecab-ko -d /path/to/dict "안녕하세요"
```

**의존성**: mecab-ko-core, clap, anyhow, serde_json

---

## 5. 빌드 및 테스트

### 5.1 빌드 명령어

```bash
# 전체 빌드
cd rust
cargo build

# 릴리스 빌드
cargo build --release

# 특정 crate 빌드
cargo build -p mecab-ko-hangul

# 문서 생성
cargo doc --open
```

### 5.2 테스트

```bash
# 전체 테스트
cargo test

# 특정 crate 테스트
cargo test -p mecab-ko-hangul

# 문서 테스트
cargo test --doc
```

### 5.3 린트 및 포맷팅

```bash
# Clippy 검사
cargo clippy --all-targets

# 포맷팅
cargo fmt

# 포맷팅 확인
cargo fmt --check
```

### 5.4 Workspace 설정

```toml
# Cargo.toml
[workspace]
resolver = "2"
members = [
    "crates/mecab-ko",
    "crates/mecab-ko-core",
    "crates/mecab-ko-dict",
    "crates/mecab-ko-dict-builder",
    "crates/mecab-ko-hangul",
    "crates/mecab-ko-cli",
]

[workspace.lints.rust]
unsafe_code = "deny"
missing_docs = "warn"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
```

---

## 6. 개발 로드맵

### Phase 1: 코어 구현 (현재)

```
Sprint 1-2:
├── ✅ DIC-001: mecab-ko-dic 소스 분석
├── ✅ DIC-002: 품사 태그 체계 검토 (pos_tag.rs)
├── ✅ RST-001: Lindera 코드베이스 분석
├── ✅ RST-002: 프로젝트 구조 설계
└── 🔄 RST-003: Lattice 구조체 구현

Sprint 3-4:
├── □ RST-004: Viterbi 알고리즘 구현
├── □ RST-005: 띄어쓰기 패널티 구현
└── □ RST-006: 사전 로더 구현
```

### Phase 2: 사전 빌더

```
Sprint 5-6:
├── □ DIC-003: CSV 파서 구현
├── □ DIC-004: Double Array Trie 빌더
└── □ DIC-005: 바이너리 포맷 정의
```

### Phase 3: 통합 및 최적화

```
Sprint 7-8:
├── □ RST-010: CLI 완성
├── □ RST-011: 벤치마크 구현
└── □ RST-012: 성능 최적화
```

### Phase 4: 바인딩

```
Sprint 9-10:
├── □ BND-001: Python 바인딩 (PyO3)
└── □ BND-002: C FFI
```

---

## 부록: 주요 의존성

| 의존성 | 버전 | 용도 |
|--------|------|------|
| thiserror | 1.0 | 에러 정의 |
| anyhow | 1.0 | 에러 전파 |
| serde | 1.0 | 직렬화 |
| fst | 0.4 | FST 자료구조 |
| yada | 0.5 | Double Array Trie |
| memmap2 | 0.9 | Memory-mapped IO |
| zstd | 0.13 | 압축 |
| rkyv | 0.8 | Zero-copy 직렬화 |
| csv | 1.3 | CSV 파싱 |
| clap | 4.4 | CLI 파싱 |
| criterion | 0.5 | 벤치마크 |
| pyo3 | 0.20 | Python 바인딩 |

---

## 참고 자료

- [docs/lindera-analysis.md](./lindera-analysis.md) - Lindera 분석 보고서
- [docs/dictionary-format-v2.md](./dictionary-format-v2.md) - 사전 포맷 문서
- [docs/pos-tag-mapping.md](./pos-tag-mapping.md) - 품사 태그 매핑
- [docs/build-process.md](./build-process.md) - 빌드 프로세스
