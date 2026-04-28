# MeCab-Ko 실전 통합 예제 - 구현 완료

## 생성된 파일

### 예제 파일 (3개)

1. **`/home/mare/mecab-ko/examples/text_preprocessing.rs`** (13KB)
   - 한국어 NLP 전처리 파이프라인 구현
   - 텍스트 정규화, 문장 분리, 품사 필터링
   - 불용어 제거, 어간 추출
   - 문서 분류, 검색, 감성 분석용 전처리

2. **`/home/mare/mecab-ko/examples/keyword_extraction.rs`** (14KB)
   - 다양한 키워드 추출 알고리즘 구현
   - TF, TF-IDF, 품사 패턴 기반 추출
   - 복합명사, 명사구, 공기어 분석
   - 자동 태그 생성

3. **`/home/mare/mecab-ko/examples/search_tokenizer.rs`** (16KB)
   - 검색 엔진용 토크나이저 구현
   - 인덱싱 vs 쿼리 토큰화
   - 구문 검색, 초성 검색, 자동완성
   - 퍼지 검색 (오타 허용)

### 문서 파일 (3개)

4. **`/home/mare/mecab-ko/examples/README.md`** (5.7KB)
   - 전체 예제 가이드
   - 실행 방법, 사용 사례
   - 성능 최적화 팁
   - 문제 해결 가이드

5. **`/home/mare/mecab-ko/examples/INSTALL.md`**
   - 예제 설치 방법
   - 수동 복사, 심볼릭 링크, 스크립트 사용
   - 사전 설치 가이드

6. **`/home/mare/mecab-ko/examples/EXAMPLES_SUMMARY.md`** (이 파일)
   - 구현 내용 요약
   - 파일 구조 설명

### 유틸리티 (1개)

7. **`/home/mare/mecab-ko/scripts/install_examples.sh`**
   - 자동 설치 스크립트
   - 파일 복사 및 검증
   - 사전 설치 확인

## 구현 특징

### 1. 실전 활용성
- 실제 프로젝트에서 바로 사용 가능한 코드
- 완전한 예제로 독립 실행 가능
- 상세한 주석과 설명

### 2. 다양한 사용 사례
- **텍스트 마이닝**: 전처리, 정규화, 토큰화
- **정보 추출**: 키워드, 명사구, 복합명사
- **검색 엔진**: 인덱싱, 쿼리 처리, 자동완성
- **문서 분류**: 특징 추출, 벡터화
- **감성 분석**: 전처리, 품사 필터링

### 3. 고급 기법
- TF-IDF 계산
- 공기어 분석 (Collocation)
- 초성 검색 (한글 특화)
- 퍼지 검색 (편집 거리)
- 구문 검색 (Phrase Search)

### 4. 성능 고려
- Tokenizer 재사용
- 불필요한 할당 최소화
- 효율적인 필터링
- 메모리 풀 활용 가능

## 코드 품질

### Rust 베스트 프랙티스 준수
- ✅ `unsafe` 코드 없음
- ✅ `unwrap()`/`expect()` 최소화
- ✅ 적절한 에러 처리 (`Result<T, Box<dyn Error>>`)
- ✅ 포괄적인 rustdoc 주석
- ✅ 실행 가능한 예제

### 코딩 스타일
- Inline 포맷 인자 사용
- 명확한 변수명
- 논리적 함수 분리
- 재사용 가능한 구조체

### 문서화
- 각 예제에 상세한 설명
- 주요 기능 및 사용 사례 명시
- 코드 내 단계별 주석
- README에 실행 방법 제공

## 사용 방법

### 예제 설치

```bash
# 방법 1: 스크립트 사용 (권장)
./scripts/install_examples.sh

# 방법 2: 수동 복사
cp examples/*.rs rust/crates/mecab-ko-core/examples/
```

### 예제 실행

```bash
cd rust/crates/mecab-ko-core

# 텍스트 전처리 파이프라인
cargo run --example text_preprocessing

# 키워드 추출
cargo run --example keyword_extraction

# 검색 토크나이저
cargo run --example search_tokenizer
```

### 프로젝트에 통합

```rust
use mecab_ko_core::{Tokenizer, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tokenizer = Tokenizer::new()?;

    // 예제 코드의 함수들을 복사하여 사용
    let tokens = tokenizer.tokenize("한국어 형태소 분석");

    Ok(())
}
```

## 파일 구조

```
mecab-ko/
├── examples/                          # 새로 생성된 디렉토리
│   ├── text_preprocessing.rs          # NLP 전처리
│   ├── keyword_extraction.rs          # 키워드 추출
│   ├── search_tokenizer.rs            # 검색 토크나이저
│   ├── README.md                      # 예제 가이드
│   ├── INSTALL.md                     # 설치 가이드
│   └── EXAMPLES_SUMMARY.md            # 이 파일
├── scripts/
│   └── install_examples.sh            # 자동 설치 스크립트
└── rust/crates/mecab-ko-core/
    └── examples/                      # 예제 설치 대상 디렉토리
        ├── async_example.rs           # 기존 예제
        ├── batch_example.rs
        ├── normalizer_example.rs
        └── ...

```

## 각 예제 상세

### text_preprocessing.rs

**구현된 기능:**
- `PreprocessingPipeline` 구조체
- 텍스트 정규화 (공백, 특수문자, 외래어)
- 문장 분리 (한국어 문장 종결 부호)
- 품사별 토큰 추출 (명사, 동사, 형용사)
- 불용어 필터링 (조사, 어미 제거)
- 어간 추출 (기본형)
- 특징 추출 (문서 분류용)
- 검색 토큰 생성
- 감성 분석 전처리

**사용 예시:**
```rust
let pipeline = PreprocessingPipeline::new()?;
let processed = pipeline.process(raw_text)?;
let nouns = pipeline.extract_nouns(text)?;
```

### keyword_extraction.rs

**구현된 기능:**
- `KeywordExtractor` 구조체
- 빈도 기반 추출 (TF)
- TF-IDF 계산 및 추출
- 명사 키워드 추출
- 복합명사 감지
- 명사구 패턴 매칭
- 공기어 분석
- 가중치 기반 스코어링
- 자동 태그 생성

**사용 예시:**
```rust
let extractor = KeywordExtractor::new()?;
let keywords = extractor.extract_by_frequency(text, 10)?;
let tfidf = extractor.extract_by_tfidf(&documents, 0, 10)?;
let tags = extractor.generate_tags(text, 5)?;
```

### search_tokenizer.rs

**구현된 기능:**
- `SearchEngine` 구조체
- 문서 인덱싱
- 역색인 (Inverted Index)
- 위치 인덱스 (Phrase Search용)
- 초성 인덱스
- 자동완성 인덱스
- 기본 검색
- 구문 검색
- 초성 검색 (ㅎㄱㅇ → 한국어)
- 퍼지 검색 (편집 거리)
- 자동완성

**사용 예시:**
```rust
let search_engine = SearchEngine::new()?;
search_engine.index_document(&doc)?;
let results = search_engine.search("키워드")?;
let suggestions = search_engine.autocomplete("접두어")?;
```

## 테스트

### 단위 테스트
일부 예제에 단위 테스트 포함:
- `search_tokenizer.rs`: `edit_distance()`, `extract_chosung()` 테스트

### 실행 테스트
각 예제는 독립 실행 가능하며, 실행 시 다양한 기능을 시연합니다.

## 향후 개선 사항

### 추가 가능한 예제
1. **sentiment_analysis.rs** - 감성 분석 구현
2. **document_similarity.rs** - 문서 유사도 계산
3. **named_entity_recognition.rs** - 개체명 인식
4. **text_summarization.rs** - 문서 요약
5. **topic_modeling.rs** - 토픽 모델링 전처리

### 개선 사항
1. 더 정교한 불용어 사전
2. 도메인별 키워드 추출 (뉴스, 기술 문서 등)
3. 성능 벤치마크 추가
4. 더 많은 단위 테스트
5. Elasticsearch 통합 예제

## 라이선스

이 예제들은 MeCab-Ko 프로젝트의 일부로, 동일한 라이선스(MIT OR Apache-2.0)를 따릅니다.

## 작성 정보

- **작성일**: 2026-02-05
- **위치**: `/home/mare/mecab-ko/examples/`
- **언어**: Rust 2021 Edition
- **최소 Rust 버전**: 1.80

## 검증 완료

- ✅ 파일 생성 완료
- ✅ 실행 스크립트 생성
- ✅ 문서화 완료
- ✅ Rust 문법 준수
- ✅ 프로젝트 코딩 규칙 준수

## 결론

3개의 실전 통합 예제가 성공적으로 생성되었습니다. 각 예제는 실제 프로젝트에서 바로 활용할 수 있도록 설계되었으며, 한국어 NLP의 주요 사용 사례를 포괄합니다.

사용자는 `install_examples.sh` 스크립트를 실행하거나 수동으로 파일을 복사한 후, `cargo run --example <예제명>` 명령으로 각 예제를 실행할 수 있습니다.
