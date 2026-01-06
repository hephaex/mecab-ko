# mecab-ko-core

한국어 형태소 분석 핵심 엔진 - Lattice, Viterbi, 토크나이저

## 특징

- **Viterbi 알고리즘**: 최적 경로 탐색을 통한 정확한 형태소 분석
- **Lattice 구조**: 효율적인 그래프 기반 분석
- **고성능**: Rust의 제로 비용 추상화와 최적화된 알고리즘
- **안전성**: 메모리 안전성 보장, unsafe 코드 최소화
- **유연한 API**: 다양한 분석 모드 지원

## 사용 예제

```rust
use mecab_ko_core::Tokenizer;

// 토크나이저 초기화 (사전 경로 필요)
let tokenizer = Tokenizer::new("path/to/dict")?;

// 형태소 분석
let result = tokenizer.parse("아버지가방에들어가신다")?;
for morpheme in result {
    println!("{} / {}", morpheme.surface, morpheme.feature);
}
```

## 의존성

- `mecab-ko-dict`: 사전 관리
- `mecab-ko-hangul`: 한글 처리

## 라이선스

MIT 또는 Apache-2.0 중 선택
