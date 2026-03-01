# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Conditional test execution with `skip_without_system_dict!` macro
- `system_dict_available()` helper function for runtime dictionary detection

### Changed
- Converted ignored doc tests to runnable examples in Elasticsearch crate

## [0.1.1] - 2026-03-01

### Added
- **CLI Enhancements**
  - `--benchmark N` option for performance testing
  - `--stats` option for analysis statistics display

- **User Dictionary**
  - `validate()` method with POS tag validation
  - `ValidationResult` and `DictionaryStats` structs
  - `remove_duplicates()` method for duplicate detection
  - `remove_surface()` method for entry removal
  - `stats()` method for dictionary statistics
  - `is_valid_pos_tag()` function with Sejong POS tag set

- **Memory Optimization**
  - `LazyEntries` struct with mmap + LRU cache
  - entries.bin v2 format with index table and O(1) random access
  - `LoadOptions` struct for configurable loading
  - `load_memory_optimized()` convenience method

- **WASM Support**
  - Made zstd compression optional (`default = ["zstd"]`)
  - WASM build now works with `default-features = false`

- **CI/CD**
  - PyPI publish workflow (maturin)
  - npm publish workflow (Node.js bindings)
  - Benchmark automation in CI

- **Performance**
  - 45-55% tokenizer improvement (10 chars: 8.6µs → 3.8µs)
  - Profiler with baseline save/compare and regression detection

### Changed
- Updated wasm-bindgen to 0.2.114
- Updated tempfile to 3.26 (workspace centralized)
- Improved README with performance metrics and examples

### Fixed
- Clippy warnings across workspace
- Rustdoc links (hyphen → underscore)

## [0.1.0] - 2026-02-15

### Added
- Initial release
- Pure Rust implementation of MeCab-Ko
- Korean morphological analysis with Viterbi algorithm
- System dictionary support (mecab-ko-dic)
- User dictionary support
- Python bindings (PyO3)
- WASM bindings (wasm-bindgen)
- Node.js bindings (N-API)
- Elasticsearch/Nori compatibility layer
- CLI tool with multiple output formats

### Performance
- ~238K morphemes/sec (mini-dict)
- 0.086ms cold start

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
