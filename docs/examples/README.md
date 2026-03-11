# MeCab-Ko 실전 활용 예제

이 디렉토리에는 MeCab-Ko를 실제 프로젝트에서 활용하는 방법을 보여주는 실전 예제들이 포함되어 있습니다.

## 예제 목록

### 1. text_preprocessing.rs - 텍스트 전처리 파이프라인

한국어 자연어 처리를 위한 완전한 텍스트 전처리 파이프라인 구현

**주요 기능:**
- 텍스트 정규화 (공백, 특수문자, 외래어)
- 문장 분리
- 품사 필터링 (명사, 동사, 형용사 추출)
- 불용어 제거
- 어간 추출 (기본형)
- 문서 분류용 특징 추출
- 감성 분석 전처리

**사용 사례:**
- 텍스트 마이닝
- 감성 분석
- 문서 분류
- 정보 추출

**실행:**
```bash
cd /home/mare/mecab-ko/rust/crates/mecab-ko-core
cargo run --example text_preprocessing
```

### 2. keyword_extraction.rs - 키워드 추출

한국어 텍스트에서 중요한 키워드를 추출하는 다양한 알고리즘 구현

**주요 알고리즘:**
- 빈도 기반 추출 (TF)
- TF-IDF (Term Frequency-Inverse Document Frequency)
- 품사 패턴 기반 추출
- 복합명사 추출
- 명사구 패턴 매칭
- 공기어 분석 (Collocation)
- 가중치 기반 키워드 스코어링

**사용 사례:**
- 문서 요약
- 자동 태그 생성
- 검색어 추천
- 토픽 모델링 전처리

**실행:**
```bash
cd /home/mare/mecab-ko/rust/crates/mecab-ko-core
cargo run --example keyword_extraction
```

### 3. search_tokenizer.rs - 검색 엔진 토크나이저

검색 엔진 인덱싱 및 쿼리 처리를 위한 토크나이저 구현

**주요 기능:**
- 문서 인덱싱 토큰화
- 검색 쿼리 토큰화
- 구문 검색 (Phrase Search)
- 초성 검색 (ㅎㄱㅇ → 한국어)
- 자동완성 (Autocomplete)
- 퍼지 검색 (오타 허용)
- 역색인 (Inverted Index) 구축

**사용 사례:**
- Elasticsearch/OpenSearch 인덱싱
- 전문 검색 (Full-text search)
- 자동완성 기능
- 검색어 추천

**실행:**
```bash
cd /home/mare/mecab-ko/rust/crates/mecab-ko-core
cargo run --example search_tokenizer
```

## 실행 방법

### 전제 조건

1. **Rust 설치**: Rust 1.70 이상 필요
2. **MeCab-Ko 사전 설치**:
   ```bash
   # Ubuntu/Debian
   sudo apt-get install mecab-ko mecab-ko-dic

   # macOS
   brew install mecab-ko mecab-ko-dic
   ```

3. **환경 변수 설정** (선택사항):
   ```bash
   export MECAB_DICDIR=/usr/local/lib/mecab/dic/mecab-ko-dic
   ```

### 개별 예제 실행

```bash
# 프로젝트 루트에서
cd rust/crates/mecab-ko-core

# 텍스트 전처리 예제
cargo run --example text_preprocessing

# 키워드 추출 예제
cargo run --example keyword_extraction

# 검색 토크나이저 예제
cargo run --example search_tokenizer
```

### 모든 예제 실행

```bash
cd rust/crates/mecab-ko-core
cargo run --example text_preprocessing
cargo run --example keyword_extraction
cargo run --example search_tokenizer
```

## 예제 코드 구조

각 예제는 다음과 같은 구조로 작성되어 있습니다:

```rust
//! # 예제 제목
//!
//! 예제 설명 및 목적
//!
//! ## 주요 기능
//! - 기능 1
//! - 기능 2
//!
//! ## 사용 사례
//! - 사례 1
//! - 사례 2

use mecab_ko_core::{Tokenizer, Token};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 초기화
    // 2. 데이터 준비
    // 3. 처리 실행
    // 4. 결과 출력

    Ok(())
}

// 헬퍼 구조체 및 함수들
```

## 프로젝트에 통합하기

### Cargo.toml에 의존성 추가

```toml
[dependencies]
mecab-ko-core = { path = "../path/to/mecab-ko/rust/crates/mecab-ko-core" }
```

또는 crates.io에 퍼블리시 후:

```toml
[dependencies]
mecab-ko-core = "0.1.0"
```

### 기본 사용 예제

```rust
use mecab_ko_core::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut tokenizer = Tokenizer::new()?;
    let tokens = tokenizer.tokenize("한국어 형태소 분석");

    for token in tokens {
        println!("{}: {}", token.surface, token.pos);
    }

    Ok(())
}
```

## 성능 최적화

### 배치 처리

대량의 텍스트를 처리할 때는 배치 처리를 사용하세요:

```rust
use mecab_ko_core::BatchTokenizer;

let batch = BatchTokenizer::new()?;
let texts = vec!["문장1", "문장2", "문장3"];
let results = batch.tokenize_batch(&texts);
```

### 비동기 처리

비동기 환경에서는 async 기능을 활용하세요:

```rust
use mecab_ko_core::AsyncTokenizer;

let tokenizer = AsyncTokenizer::new().await?;
let tokens = tokenizer.tokenize_async("텍스트").await?;
```

## 추가 자료

- **API 문서**: `/home/mare/mecab-ko/rust/crates/mecab-ko-core/README.md`
- **프로젝트 계획**: `/home/mare/mecab-ko/docs/PROJECT_PLAN.md`
- **이슈 백로그**: `/home/mare/mecab-ko/docs/ISSUE_BACKLOG.md`
- **기존 예제**: `/home/mare/mecab-ko/rust/crates/mecab-ko-core/examples/`

## 라이선스

이 예제들은 MeCab-Ko 프로젝트의 일부로, 동일한 라이선스를 따릅니다.

## 기여하기

새로운 예제를 추가하거나 기존 예제를 개선하고 싶다면:

1. 이슈를 생성하여 제안 내용을 공유
2. Pull Request를 생성
3. 코드 리뷰 및 피드백 반영

## 문제 해결

### 사전을 찾을 수 없다는 오류

```
Error: Dictionary error: Cannot find dictionary
```

**해결 방법:**
```bash
# 사전 경로 확인
mecab-config --dicdir

# 환경 변수 설정
export MECAB_DICDIR=$(mecab-config --dicdir)
```

### 컴파일 오류

```
error: linking with `cc` failed
```

**해결 방법:**
```bash
# 개발 도구 설치
sudo apt-get install build-essential

# macOS
xcode-select --install
```

## 연락처

- GitHub Issues: [mecab-ko 이슈 트래커](https://github.com/your-repo/mecab-ko/issues)
- 문서: `/home/mare/mecab-ko/docs/`
