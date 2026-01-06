# mecab-ko-python

Python bindings for MeCab-Ko (Korean morphological analyzer)

## Overview

This package provides Python bindings for MeCab-Ko, a Korean morphological analyzer implemented in Rust. The API is compatible with KoNLPy's Mecab interface, providing high-performance Korean morphological analysis with a familiar API.

## Features

- **Fast**: Rust-based implementation with zero-copy parsing
- **Memory-efficient**: Optimized data structures for Korean text processing
- **Thread-safe**: Safe concurrent operations
- **KoNLPy-compatible**: Drop-in replacement for KoNLPy's Mecab
- **Type hints**: Full type annotation support for better IDE integration

## Installation

### From PyPI (Recommended)

```bash
pip install mecab-ko-python
```

Pre-built wheels are available for:
- Linux (x86_64, aarch64)
- macOS (x86_64, Apple Silicon)
- Windows (x86_64)

### From Source

If you need to build from source:

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin
pip install maturin

# Build and install
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust/crates/mecab-ko-python
maturin develop --release
```

## Usage

```python
from mecab_ko import Mecab

# Create tokenizer instance
mecab = Mecab()

# Extract morphemes
morphemes = mecab.morphs("안녕하세요")
print(morphemes)
# ['안녕', '하', '세요']

# Extract nouns
nouns = mecab.nouns("아버지가방에들어가신다")
print(nouns)
# ['아버지', '가방']

# Part-of-speech tagging
tagged = mecab.pos("나는 학생입니다")
print(tagged)
# [('나', 'NP'), ('는', 'JX'), ('학생', 'NNG'), ('이', 'VCP'), ('ㅂ니다', 'EF')]

# MeCab format output
result = mecab.parse("안녕하세요")
print(result)
# 안녕    NNG,*,*,안녕,*,*,*,*
# 하      XSV,*,*,하,*,*,*,*
# 세요    EF,*,*,세요,*,*,*,*
# EOS
```

## API Reference

### `Mecab(dicpath=None)`

Create a new Mecab tokenizer instance.

**Parameters:**
- `dicpath` (str, optional): Path to dictionary directory

**Returns:**
- `Mecab`: Tokenizer instance

### `mecab.morphs(text)`

Extract morphemes from text.

**Parameters:**
- `text` (str): Input text

**Returns:**
- `list[str]`: List of morphemes

### `mecab.nouns(text)`

Extract nouns from text.

**Parameters:**
- `text` (str): Input text

**Returns:**
- `list[str]`: List of nouns

### `mecab.pos(text)`

Perform part-of-speech tagging.

**Parameters:**
- `text` (str): Input text

**Returns:**
- `list[tuple[str, str]]`: List of (surface, pos_tag) tuples

### `mecab.parse(text)`

Parse text and return MeCab format output.

**Parameters:**
- `text` (str): Input text

**Returns:**
- `str`: MeCab format string with tab-separated values

## Korean POS Tags

The analyzer uses the Sejong POS tag set:

- `NNG`: General noun (일반 명사)
- `NNP`: Proper noun (고유 명사)
- `NP`: Pronoun (대명사)
- `VV`: Verb (동사)
- `VA`: Adjective (형용사)
- `JX`: Auxiliary particle (보조사)
- `JKS`: Subject particle (주격조사)
- `JKO`: Object particle (목적격조사)
- `EF`: Final ending (종결어미)
- And many more...

## Performance

The Rust implementation provides significant performance improvements over the original C++ implementation:

- Fast tokenization with zero-copy parsing
- Memory-efficient data structures
- Thread-safe operations

## Migration from KoNLPy

If you're currently using KoNLPy's Mecab, you can migrate with minimal changes:

```python
# Before (KoNLPy)
from konlpy.tag import Mecab
mecab = Mecab()

# After (mecab-ko-python)
from mecab_ko import Mecab
mecab = Mecab()

# The API is identical
mecab.morphs("안녕하세요")
mecab.nouns("아버지가방에들어가신다")
mecab.pos("나는 학생입니다")
```

## Development

```bash
# Build in development mode
maturin develop

# Run Rust tests
cargo test

# Run Python tests
pytest

# Build release wheel
maturin build --release

# Lint and format
cargo clippy
cargo fmt
```

## Publishing to PyPI

This package uses GitHub Actions for automated publishing to PyPI. To publish a new version:

1. Update the version in `Cargo.toml` and `pyproject.toml`
2. Create a new git tag: `git tag v0.1.0 && git push origin v0.1.0`
3. GitHub Actions will automatically build wheels and publish to PyPI

## License

This project is licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## References

- [MeCab](https://taku910.github.io/mecab/)
- [KoNLPy](https://konlpy.org/)
- [PyO3](https://pyo3.rs/)
