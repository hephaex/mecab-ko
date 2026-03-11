# Quick Start Guide - MeCab-Ko 실전 예제

## 빠른 시작

### 1단계: 예제 설치

```bash
# 프로젝트 루트에서
./scripts/install_examples.sh
```

### 2단계: 예제 실행

```bash
cd rust/crates/mecab-ko-core

# 예제 1: 텍스트 전처리
cargo run --example text_preprocessing

# 예제 2: 키워드 추출
cargo run --example keyword_extraction

# 예제 3: 검색 토크나이저
cargo run --example search_tokenizer
```

## 예제별 핵심 기능

### 📝 text_preprocessing.rs
```rust
// 텍스트 정규화
let normalized = pipeline.normalize_text(raw_text);

// 명사 추출
let nouns = pipeline.extract_nouns(text)?;

// 불용어 제거
let filtered = pipeline.remove_stopwords(text)?;
```

**용도**: 텍스트 마이닝, 감성 분석, 문서 분류

---

### 🔑 keyword_extraction.rs
```rust
// TF 기반 키워드
let keywords = extractor.extract_by_frequency(text, 10)?;

// TF-IDF 키워드
let tfidf = extractor.extract_by_tfidf(&docs, 0, 10)?;

// 자동 태그 생성
let tags = extractor.generate_tags(text, 5)?;
```

**용도**: 문서 요약, 태그 생성, 토픽 모델링

---

### 🔍 search_tokenizer.rs
```rust
// 문서 인덱싱
search_engine.index_document(&doc)?;

// 검색
let results = search_engine.search("키워드")?;

// 초성 검색
let results = search_engine.chosung_search("ㅎㄱㅇ")?;

// 자동완성
let suggestions = search_engine.autocomplete("접두")?;
```

**용도**: 검색 엔진, 자동완성, 전문 검색

---

## 프로젝트에 통합

### Cargo.toml
```toml
[dependencies]
mecab-ko-core = { path = "../path/to/rust/crates/mecab-ko-core" }
```

### 기본 사용
```rust
use mecab_ko_core::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tokenizer = Tokenizer::new()?;
    let tokens = tokenizer.tokenize("한국어 형태소 분석");

    for token in tokens {
        println!("{} / {}", token.surface, token.pos);
    }

    Ok(())
}
```

## 문제 해결

### 사전을 찾을 수 없음
```bash
export MECAB_DICDIR=$(mecab-config --dicdir)
```

### 컴파일 오류
```bash
# Ubuntu/Debian
sudo apt-get install build-essential mecab-ko mecab-ko-dic

# macOS
brew install mecab-ko mecab-ko-dic
```

## 더 알아보기

- **상세 가이드**: `README.md`
- **설치 방법**: `INSTALL.md`
- **구현 요약**: `EXAMPLES_SUMMARY.md`

## 예제 파일

| 파일 | 크기 | 설명 |
|------|------|------|
| `text_preprocessing.rs` | 13KB | NLP 전처리 파이프라인 |
| `keyword_extraction.rs` | 14KB | 키워드 추출 알고리즘 |
| `search_tokenizer.rs` | 16KB | 검색 엔진 토크나이저 |

## 핵심 개념

### 품사 태그 (POS Tags)
- `NN*`: 명사 (NNG, NNP, NNB)
- `VV`: 동사
- `VA`: 형용사
- `JK*`: 조사
- `SL`: 외래어

### 전처리 단계
1. 텍스트 정규화
2. 토큰화
3. 품사 태깅
4. 필터링 (불용어, 품사)
5. 정규화 (어간 추출)

### 검색 최적화
1. 인덱싱 토큰화 (모든 형태소)
2. 쿼리 토큰화 (주요 내용어)
3. 역색인 구축
4. 효율적 검색

---

**작성일**: 2026-02-05
**위치**: `/home/mare/mecab-ko/examples/`
