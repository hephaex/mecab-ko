# Changelog

All notable changes to mecab-ko-python will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- PyPI distribution configuration
- GitHub Actions workflow for automated publishing
- Type stubs for better IDE support (py.typed, __init__.pyi)
- Comprehensive documentation for PyPI releases

## [0.1.0] - 2024-01-05

### Added
- Initial release of mecab-ko-python
- KoNLPy-compatible API with `Mecab` class
- Core methods: `morphs()`, `nouns()`, `pos()`, `parse()`
- Support for custom dictionary paths
- Rust-based implementation using PyO3
- Cross-platform support (Linux, macOS, Windows)
- Python 3.8+ compatibility
- Comprehensive documentation and examples
- Type hints for better IDE integration

### Features
- Fast morphological analysis with Rust backend
- Zero-copy parsing for performance
- Thread-safe operations
- Memory-efficient data structures
- Compatible with KoNLPy's Mecab interface

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
