# 🦀 MeCab-Ko Rust Implementation

[![Crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko)
[![Documentation](https://docs.rs/mecab-ko/badge.svg)](https://docs.rs/mecab-ko)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue)](https://www.rust-lang.org/)

**고성능 한국어 형태소 분석기 - MeCab-Ko의 순수 Rust 구현**

## 📦 Crates 구조

```
rust/
├── crates/
│   ├── mecab-ko-core/      # 핵심 분석 엔진
│   │   ├── lattice/        # Lattice 구조
│   │   ├── viterbi/        # Viterbi 알고리즘
│   │   └── tokenizer/      # 토크나이저
│   │
│   ├── mecab-ko-dict/      # 사전 관리
│   │   ├── format/         # 바이너리 포맷
│   │   ├── builder/        # 사전 빌더
│   │   └── loader/         # 사전 로더
│   │
│   ├── mecab-ko-hangul/    # 한글 유틸리티
│   │   ├── jamo/           # 자모 처리
│   │   ├── syllable/       # 음절 처리
│   │   └── normalize/      # 정규화
│   │
│   └── mecab-ko-cli/       # CLI 도구
│       └── main.rs
│
└── Cargo.toml              # Workspace 설정
```

## 🚀 사용법

### 라이브러리

```toml
[dependencies]
mecab-ko = "0.1"
```

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let tokenizer = Tokenizer::new()?;
    
    // 기본 토큰화
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");
    
    for token in tokens {
        println!("{}\t{}\t{}", token.surface, token.pos, token.reading);
    }
    
    // wakati (분리만)
    let words = tokenizer.wakati("한국어 형태소 분석");
    println!("{}", words.join(" "));  // "한국어 형태소 분석"
    
    Ok(())
}
```

### CLI

```bash
# 빌드
cargo build --release --bin mecab

# 기본 분석
./target/release/mecab "안녕하세요"

# 파이프 입력
echo "형태소 분석" | ./target/release/mecab

# wakati 모드
./target/release/mecab -O wakati "한국어 처리"

# 사전 경로 지정
./target/release/mecab -d /path/to/dict "텍스트"
```

## 🔧 개발

### 빌드

```bash
# 전체 빌드
cargo build

# 릴리스 빌드
cargo build --release

# 특정 crate만
cargo build -p mecab-ko-hangul
```

### 테스트

```bash
# 전체 테스트
cargo test

# 특정 crate
cargo test -p mecab-ko-hangul

# 문서 테스트
cargo test --doc
```

### 벤치마크

```bash
cargo bench
```

### 문서 생성

```bash
cargo doc --no-deps --open
```

## 📊 Crate 의존성

```
mecab-ko-cli
    └── mecab-ko-core
            ├── mecab-ko-dict
            │       └── mecab-ko-hangul
            └── mecab-ko-hangul
```

## 🎯 설계 원칙

### 안전성 우선
- `unsafe` 코드 최소화 (SAFETY 주석 필수)
- `unwrap()`, `expect()` 금지 (라이브러리 코드)
- 모든 에러는 `Result`/`Option`으로 처리

### 성능
- Zero-copy 파싱 (가능한 경우)
- Memory-mapped 사전 로딩
- SIMD 최적화 (한글 처리)

### 호환성
- 기존 mecab-ko-dic 포맷 지원
- konlpy 호환 API (Python 바인딩)
- MeCab 출력 포맷 호환

## 📝 각 Crate 설명

### mecab-ko-hangul
한글 처리를 위한 기초 유틸리티:
- 자모 분리/결합
- 음절 유효성 검사
- 종성(받침) 판별
- 한글 정규화

### mecab-ko-dict
사전 관리 시스템:
- 바이너리 사전 포맷 (v3.0)
- FST 기반 형태소 검색
- 연접 비용 매트릭스
- 사전 빌더/컴파일러

### mecab-ko-core
핵심 분석 엔진:
- Lattice 구축
- Viterbi 알고리즘
- N-best 경로 탐색
- 미등록어 처리

### mecab-ko-cli
명령줄 인터페이스:
- 다양한 출력 포맷
- 파이프라인 처리
- 배치 모드

## 🔗 관련 링크

- [프로젝트 메인](https://github.com/hephaex/mecab-ko)
- [Legacy C++ 구현](https://github.com/hephaex/mecab-ko/tree/main/legacy)
- [프로젝트 계획](https://github.com/hephaex/mecab-ko/blob/main/docs/PROJECT_PLAN.md)
- [이슈 백로그](https://github.com/hephaex/mecab-ko/blob/main/docs/ISSUE_BACKLOG.md)
