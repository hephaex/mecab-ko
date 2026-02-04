# MeCab-Ko

[![CI](https://github.com/hephaex/mecab-ko/actions/workflows/ci.yml/badge.svg)](https://github.com/hephaex/mecab-ko/actions/workflows/ci.yml)
[![Security](https://github.com/hephaex/mecab-ko/actions/workflows/security.yml/badge.svg)](https://github.com/hephaex/mecab-ko/actions/workflows/security.yml)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![crates.io](https://img.shields.io/crates/v/mecab-ko.svg)](https://crates.io/crates/mecab-ko)
[![PyPI](https://img.shields.io/pypi/v/mecab-ko-python.svg)](https://pypi.org/project/mecab-ko-python/)
[![npm](https://img.shields.io/npm/v/mecab-ko-wasm.svg)](https://www.npmjs.com/package/mecab-ko-wasm)

**High-performance Korean morphological analyzer written in pure Rust**

MeCab-Ko is a modern reimplementation of the [MeCab-Ko](https://bitbucket.org/eunjeon/mecab-ko) Korean morphological analyzer, originally developed by the Eunjeon project. This Rust implementation provides memory safety, cross-platform support (including WebAssembly), and bindings for Python, Node.js, and browsers.

## Features

- **Pure Rust Implementation** - Memory-safe with `#![deny(unsafe_code)]`
- **High Performance** - Zero-copy parsing, efficient Viterbi algorithm (~150K words/sec)
- **Cross-Platform** - Linux, macOS, Windows, and WebAssembly
- **Multiple Bindings** - Python (PyO3), Node.js (N-API), WebAssembly
- **Korean Optimized** - Space penalty handling, Jamo processing, Jongseong-based rules
- **User Dictionary** - Custom word support with hot-reload capability
- **KoNLPy Compatible** - Drop-in replacement for KoNLPy's Mecab API

## Quick Start

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
mecab-ko = "0.1"
```

```rust
use mecab_ko::Tokenizer;

fn main() -> Result<(), mecab_ko::Error> {
    let tokenizer = Tokenizer::new()?;

    // Tokenize Korean text
    let tokens = tokenizer.tokenize("안녕하세요, 형태소 분석기입니다.");
    for token in tokens {
        println!("{}\t{}", token.surface, token.pos);
    }
    // Output:
    // 안녕    NNG
    // 하      XSV
    // 세요    EF
    // ...

    // Wakati mode (space-separated morphemes)
    let words = tokenizer.wakati("한국어 형태소 분석");
    println!("{}", words.join(" "));

    // Extract nouns only
    let nouns = tokenizer.nouns("오늘 날씨가 좋습니다");
    println!("{:?}", nouns);  // ["오늘", "날씨"]

    Ok(())
}
```

### Python

```bash
pip install mecab-ko-python
```

```python
from mecab_ko import Mecab

mecab = Mecab()

# Extract morphemes
print(mecab.morphs("안녕하세요"))
# ['안녕', '하', '세요']

# Extract nouns
print(mecab.nouns("아버지가방에들어가신다"))
# ['아버지', '가방']

# Part-of-speech tagging
print(mecab.pos("나는 학생입니다"))
# [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]

# MeCab format output
print(mecab.parse("형태소"))
# 형태소  NNG,*,*,*,*,*,*,*
# EOS
```

### Node.js

```bash
npm install @mecab-ko/node
```

```javascript
const { Mecab } = require('@mecab-ko/node');

const mecab = new Mecab();

// Tokenize text
const tokens = mecab.tokenize('형태소 분석기');
console.log(tokens);
// [{ surface: '형태소', pos: 'NNG', ... }, { surface: '분석기', pos: 'NNG', ... }]

// Extract morphemes
const morphs = mecab.morphs('안녕하세요');
console.log(morphs);  // ['안녕', '하', '세요']

// Extract nouns
const nouns = mecab.nouns('대한민국의 수도는 서울입니다');
console.log(nouns);  // ['대한민국', '수도', '서울']

// POS tagging
const pos = mecab.pos('좋은 아침입니다');
console.log(pos);  // [['좋은', 'VA+ETM'], ['아침', 'NNG'], ['입니다', 'VCP+EF']]
```

### WebAssembly (Browser)

```bash
npm install mecab-ko-wasm
```

```javascript
import init, { Mecab } from 'mecab-ko-wasm';

async function analyze() {
    // Initialize WASM module
    await init();
    const mecab = new Mecab();

    // Extract morphemes
    const morphs = mecab.morphs("안녕하세요");
    console.log(morphs);  // ["안녕", "하", "세요"]

    // Extract nouns
    const nouns = mecab.nouns("형태소 분석기입니다");
    console.log(nouns);  // ["형태소", "분석기"]

    // Get detailed token information
    const tokens = mecab.tokenize("한국어 분석기");
    tokens.forEach(token => {
        console.log(`${token.surface}: ${token.pos}`);
    });
}

analyze();
```

## Crate Structure

MeCab-Ko is organized as a Cargo workspace with the following crates:

| Crate | Description |
|-------|-------------|
| `mecab-ko` | Integration crate re-exporting all public APIs |
| `mecab-ko-core` | Core tokenization engine (Lattice, Viterbi, Tokenizer) |
| `mecab-ko-dict` | Dictionary management (Trie, Matrix, User Dictionary) |
| `mecab-ko-hangul` | Korean Hangul utilities (Jamo decomposition/composition) |
| `mecab-ko-cli` | Command-line interface |
| `mecab-ko-dict-builder` | Dictionary compilation tools |
| `mecab-ko-python` | Python bindings (PyO3) |
| `mecab-ko-node` | Node.js bindings (N-API) |
| `mecab-ko-wasm` | WebAssembly bindings |
| `mecab-ko-elasticsearch` | Elasticsearch/Lucene integration |
| `mecab-ko-profiler` | Performance profiling tools |

## Build Instructions

### Prerequisites

- Rust 1.75 or later
- For Python bindings: Python 3.8+ and maturin
- For Node.js bindings: Node.js 14+ and npm
- For WASM: wasm-pack

### Building from Source

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust

# Build all crates
cargo build --release

# Run tests
cargo test

# Run linter
cargo clippy

# Format code
cargo fmt

# Build documentation
cargo doc --no-deps --open
```

### Building Python Bindings

```bash
cd rust/crates/mecab-ko-python
pip install maturin
maturin develop --release
```

### Building Node.js Bindings

```bash
cd rust/crates/mecab-ko-node
npm install
npm run build
```

### Building WASM

```bash
cd rust/crates/mecab-ko-wasm
cargo install wasm-pack
rustup target add wasm32-unknown-unknown
wasm-pack build --target web --release
```

## Documentation

- **[Architecture Guide](docs/ARCHITECTURE.md)** - System design, component interactions, and data flow
- **[Getting Started](docs/RUST_GETTING_STARTED.md)** - Comprehensive installation and usage guide
- **[Project Plan](docs/PROJECT_PLAN.md)** - Development roadmap

## Common POS Tags

MeCab-Ko uses the Sejong corpus POS tag set:

| Tag | Korean | English |
|-----|--------|---------|
| NNG | 일반 명사 | General noun |
| NNP | 고유 명사 | Proper noun |
| NNB | 의존 명사 | Dependent noun |
| NP | 대명사 | Pronoun |
| VV | 동사 | Verb |
| VA | 형용사 | Adjective |
| VX | 보조 용언 | Auxiliary verb |
| MAG | 일반 부사 | General adverb |
| JKS | 주격 조사 | Subject particle |
| JKO | 목적격 조사 | Object particle |
| JX | 보조사 | Auxiliary particle |
| EF | 종결 어미 | Final ending |
| SF | 마침표 | Period |

## Performance Targets

| Metric | Target |
|--------|--------|
| Tokenization Speed | ~150K words/sec |
| Accuracy | ~95% |
| Memory Usage | ~150MB |
| WASM Module Size | ~2-5MB |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Development workflow
cd rust
cargo build          # Build
cargo test           # Test
cargo clippy         # Lint
cargo fmt            # Format
```

### Coding Rules

- `unsafe` code is prohibited (`#![deny(unsafe_code)]`)
- `unwrap()` and `expect()` are prohibited in library code
- All public APIs must have rustdoc documentation

## License

This project is dual-licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

Choose the license that best fits your needs.

Dictionary data is licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

## Acknowledgments

- [MeCab](https://taku910.github.io/mecab/) - Taku Kudo
- [Eunjeon Project](https://bitbucket.org/eunjeon/mecab-ko) - Original mecab-ko
- [Lindera](https://github.com/lindera/lindera) - Rust morphological analyzer reference
- [Kiwi](https://github.com/bab2min/Kiwi) - Korean morphological analyzer reference
- [KoNLPy](https://konlpy.org/) - Python Korean NLP library

## Contact

- **Author**: hephaex (hephaex@gmail.com)
- **Issues**: [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- **Discussions**: [GitHub Discussions](https://github.com/hephaex/mecab-ko/discussions)
