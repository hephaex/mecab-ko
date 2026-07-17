# 🦀 MeCab-Ko Rust Implementation

[![Crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko)
[![Documentation](https://docs.rs/mecab-ko/badge.svg)](https://docs.rs/mecab-ko)
[![MSRV](https://img.shields.io/badge/MSRV-1.83-blue)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

**고성능 한국어 형태소 분석기 - MeCab-Ko의 순수 Rust 구현**

## Features

- **순수 Rust**: C/C++ 의존성 없음, 모든 플랫폼 지원
- **고성능**: 238K morphemes/sec, 0.086ms cold start
- **메모리 효율**: mmap 기반 사전 로딩, lazy loading 지원
- **다양한 바인딩**: Python (PyO3), WASM, Node.js
- **MeCab 호환**: 기존 mecab-ko-dic 사전 형식 지원

## 📦 Installation

### Rust

```toml
[dependencies]
mecab-ko = "0.7.2"
```

### Python

```bash
pip install mecab-ko-python
```

### WASM (npm)

```bash
npm install mecab-ko-wasm
```

## 🚀 Quick Start

### Rust

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let tokenizer = Tokenizer::new()?;

    // 기본 토큰화
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");

    for token in tokens {
        println!("{}\t{}", token.surface, token.pos);
    }

    // wakati (분리만)
    let words = tokenizer.wakati("한국어 형태소 분석");
    println!("{}", words.join(" "));  // "한국어 형태소 분석"

    Ok(())
}
```

### Python

```python
from mecab_ko import Mecab

mecab = Mecab()

# 형태소 분석
result = mecab.parse("안녕하세요")
for morpheme in result:
    print(f"{morpheme.surface}\t{morpheme.pos}")

# KoNLPy 호환 API
nouns = mecab.nouns("한국어 자연어 처리")
print(nouns)  # ['한국어', '자연어', '처리']
```

### CLI

```bash
# 빌드
cargo build --release --bin mecab

# 기본 분석
./target/release/mecab "안녕하세요"

# wakati 모드
./target/release/mecab -O wakati "한국어 처리"

# 사전 경로 지정
./target/release/mecab -d /path/to/dict "텍스트"
```

## 📦 Crates 구조

```
rust/crates/
├── mecab-ko/               # 통합 파사드 API
├── mecab-ko-core/          # 핵심 분석 엔진 (Lattice, Viterbi)
├── mecab-ko-dict/          # 사전 관리 (Trie, Matrix, Loader)
├── mecab-ko-hangul/        # 한글 유틸리티 (자모, 음절)
├── mecab-ko-dict-builder/  # 사전 빌드 도구 (CSV → binary)
├── mecab-ko-dict-validator/# 사전 검증 도구
├── mecab-ko-cli/           # CLI 인터페이스
├── mecab-ko-python/        # Python 바인딩 (PyO3)
├── mecab-ko-wasm/          # WASM 바인딩
├── mecab-ko-node/          # Node.js 바인딩
├── mecab-ko-elasticsearch/ # Elasticsearch Nori 호환
└── mecab-ko-profiler/      # 성능 프로파일러
```

## 📊 Performance

| Metric | Value | Target |
|--------|-------|--------|
| Throughput | 238K morphemes/sec | 150K |
| Cold Start | 0.086ms | < 200ms |
| Memory (full dict) | 215MB | < 150MB |

*벤치마크: Apple M1, mecab-ko-dic 2.1.1*

## 🔧 Development

### Build

```bash
# 전체 빌드
cargo build

# 릴리스 빌드
cargo build --release

# 특정 crate만
cargo build -p mecab-ko-hangul
```

### Test

```bash
# 전체 테스트
cargo test

# 특정 crate
cargo test -p mecab-ko-core

# 문서 테스트
cargo test --doc
```

### Benchmark

```bash
cargo bench
```

## 🎯 Design Principles

- **안전성**: `unsafe` 최소화, `unwrap()` 금지 (라이브러리)
- **성능**: Zero-copy 파싱, mmap 사전 로딩
- **호환성**: MeCab 출력 포맷, KoNLPy API 호환

## 📝 Crate 상세

### mecab-ko-hangul
한글 처리 유틸리티: 자모 분리/결합, 음절 검사, 종성 판별

### mecab-ko-dict
사전 관리: 바이너리 포맷 v3.0, FST 검색, 연접 비용 매트릭스, Hot Reload

### mecab-ko-core
분석 엔진: Lattice 구축, Viterbi 최적 경로, N-best 탐색, 미등록어 처리

## 🔗 Links

- [GitHub Repository](https://github.com/hephaex/mecab-ko)
- [Documentation](https://docs.rs/mecab-ko)
- [Project Plan](https://github.com/hephaex/mecab-ko/blob/main/docs/PROJECT_PLAN.md)
- [Legacy C++ Implementation](https://github.com/hephaex/mecab-ko/tree/main/legacy)

## License

MIT OR Apache-2.0
