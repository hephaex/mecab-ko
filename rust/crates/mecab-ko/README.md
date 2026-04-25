# mecab-ko

[![Crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko)
[![Documentation](https://docs.rs/mecab-ko/badge.svg)](https://docs.rs/mecab-ko)
[![License](https://img.shields.io/crates/l/mecab-ko.svg)](https://github.com/hephaex/mecab-ko)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue)](https://www.rust-lang.org/)

**고성능 한국어 형태소 분석기 - MeCab-Ko의 순수 Rust 구현**

🎉 **v0.7.1: 1,181 테스트 100% 통과, Streaming/Domain Overlay/Search Plugin 추가!**

MeCab-Ko는 일본어 형태소 분석기 MeCab을 한국어에 맞게 개선한 형태소 분석 도구입니다. 이 크레이트는 기존 C++ 구현을 Rust로 재작성하여 메모리 안전성과 성능을 동시에 달성합니다.

## 특징

- **메모리 안전성**: Rust의 소유권 시스템으로 메모리 버그 방지
- **고성능**: 제로 비용 추상화와 최적화된 알고리즘
- **순수 Rust**: C/C++ 의존성 없이 크로스 플랫폼 지원
- **MeCab 호환**: 기존 mecab-ko-dic 사전 포맷 지원
- **사용자 사전**: 도메인별 용어 추가 가능

## 설치

Cargo.toml에 추가:

```toml
[dependencies]
mecab-ko = "0.7.1"
```

## 빠른 시작

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    // 토크나이저 초기화
    let tokenizer = Tokenizer::new()?;

    // 기본 형태소 분석
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");

    for token in tokens {
        println!("{}\t{}\t{}", token.surface, token.pos, token.reading);
    }

    Ok(())
}
```

## 주요 기능

### 기본 토큰화

```rust
let tokens = tokenizer.tokenize("한국어 형태소 분석");
for token in tokens {
    println!("{} / {}", token.surface, token.pos);
}
// 출력:
// 한국어 / NNG
// 형태소 / NNG
// 분석 / NNG
```

### Wakati 모드 (어절 분리)

```rust
let words = tokenizer.wakati("한국어 형태소 분석");
println!("{}", words.join(" "));
// 출력: "한국어 형태소 분석"
```

### 사용자 사전

```rust
use mecab_ko::{Tokenizer, UserDictionaryBuilder};

// 사용자 사전 생성
let user_dict = UserDictionaryBuilder::new()
    .add("딥러닝", "NNG")
    .add("챗GPT", "NNP")
    .build()?;

// 토크나이저에 적용
let tokenizer = Tokenizer::with_user_dict(user_dict)?;

let tokens = tokenizer.tokenize("딥러닝과 챗GPT");
```

### 사용자 사전 검증 및 통계

```rust
use mecab_ko::dict::UserDictionary;

// CSV 파일에서 로드
let mut dict = UserDictionary::load_from_csv("user_dict.csv")?;

// 사전 검증 (빈 항목, 유효하지 않은 품사 태그 검사)
let result = dict.validate();
println!("유효: {}", result.is_valid());
println!("에러: {:?}", result.errors);
println!("경고: {:?}", result.warnings);

// 중복 항목 제거
let removed = dict.remove_duplicates();
println!("제거된 중복 항목: {}", removed);

// 사전 통계
let stats = dict.stats();
println!("{}", stats);  // 항목 수, 품사 분포, 평균 비용 출력
```

### 사전 경로 지정

```rust
let tokenizer = Tokenizer::with_dict_path("/path/to/mecab-ko-dic")?;
```

## 크레이트 구조

mecab-ko는 다음 하위 크레이트들로 구성됩니다:

- **[mecab-ko-core](https://crates.io/crates/mecab-ko-core)**: 핵심 분석 엔진 (Lattice, Viterbi)
- **[mecab-ko-dict](https://crates.io/crates/mecab-ko-dict)**: 사전 관리 및 로딩
- **[mecab-ko-hangul](https://crates.io/crates/mecab-ko-hangul)**: 한글 처리 유틸리티

## 성능

Rust 구현은 기존 C++ 구현과 비슷하거나 더 나은 성능을 제공합니다:

| 지표 | 측정값 | 비고 |
|-----|-------|-----|
| 처리 속도 | ~238K 형태소/초 | mini-dict 기준 |
| Cold start | 0.086ms | mmap 사용 시 |
| 메모리 (Full) | ~215 MB | mecab-ko-dic 2.1.1 |
| 메모리 (최적화) | ~150 MB | LazyEntries + mmap |

### 성능 개선 (v0.1.1)

| 입력 길이 | 개선 전 | 개선 후 | 개선율 |
|----------|--------|--------|-------|
| 10자 | 8.6µs | 3.8µs | -55% |
| 100자 | 198µs | 141µs | -31% |
| 1000자 | 9978µs | 8413µs | -16% |

## 최소 Rust 버전 (MSRV)

이 크레이트는 Rust 1.75 이상을 요구합니다.

## 라이선스

MIT OR Apache-2.0 중 선택

## 기여

이슈와 풀 리퀘스트는 언제나 환영합니다!

## 참고 자료

- [프로젝트 저장소](https://github.com/hephaex/mecab-ko)
- [Legacy C++ 구현](https://bitbucket.org/eunjeon/mecab-ko)
- [mecab-ko-dic 사전](https://bitbucket.org/eunjeon/mecab-ko-dic)

## 관련 프로젝트

- [mecab](https://taku910.github.io/mecab/) - 원본 일본어 형태소 분석기
- [konlpy](https://konlpy.org/) - 한국어 NLP 파이썬 라이브러리
- [elasticsearch-analysis-nori](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html) - Elasticsearch 한국어 분석기
