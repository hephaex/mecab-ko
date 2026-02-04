# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Core Library (mecab-ko-core)
- Viterbi algorithm implementation for optimal path search in morphological analysis
- Lattice structure for efficient graph-based analysis
- Core tokenizer with Korean-specific optimizations
- High-performance Rust-based morphological analysis engine
- Zero-cost abstractions for maximum performance
- Memory-safe implementation with no `unsafe` code in library

#### Hangul Utilities (mecab-ko-hangul)
- Jamo decomposition/composition functions (syllable to initial/medial/final consonants)
- Syllable processing and validation
- Korean character type classification
- Final consonant (jongseong) detection
- Unicode normalization for Korean text
- Zero external dependencies for lightweight deployment

#### Dictionary Management (mecab-ko-dict)
- Double-Array Trie implementation for fast dictionary lookup
- Connection cost matrix loader and handler
- User dictionary support (CSV format)
- Binary dictionary format v3.0 design
- Memory-mapped file support for large dictionaries
- Dictionary validation tools

#### Command Line Interface (mecab-ko-cli)
- Multiple output formats: default, wakati, json, csv, pos, simple, dump
- User dictionary support via command line
- N-best output for multiple analysis candidates
- Batch processing support

#### Python Bindings (mecab-ko-python)
- KoNLPy-compatible API with `Mecab` class
- Core methods: `morphs()`, `nouns()`, `pos()`, `parse()`
- Custom dictionary path support
- Cross-platform support (Linux, macOS, Windows)
- Python 3.8+ compatibility
- Type stubs for IDE support (py.typed, __init__.pyi)
- PyPI distribution configuration
- GitHub Actions workflow for automated publishing

#### Node.js Bindings (mecab-ko-node)
- Native Node.js bindings using napi-rs
- `Mecab` class with `tokenize()`, `morphs()`, `nouns()`, `pos()` methods
- `getVersion()` function for library version
- Full TypeScript type definitions (.d.ts)
- Cross-platform support (macOS, Linux, Windows - x64/ARM64)
- Zero-copy operations for performance
- Thread-safe implementation

#### WebAssembly (mecab-ko-wasm)
- Browser and Node.js WASM support
- Near-native performance through WebAssembly compilation
- Full TypeScript type definitions
- Methods: `tokenize()`, `morphs()`, `nouns()`, `pos()`, `wakati()`
- ~1-2ms tokenization for typical sentences
- ~2-5MB WASM module size with dictionary

#### Elasticsearch Integration (mecab-ko-elasticsearch)
- Lucene Nori-compatible analyzer (`NoriAnalyzer`)
- Token filters: `NoriPartOfSpeechStopFilter`, `NoriReadingFormFilter`
- Composite filter support for chaining
- Decompound modes: none, discard, mixed
- Stoptags configuration for POS filtering
- JNI bindings for Java/Elasticsearch integration (feature-gated)
- Integration tests and benchmarks

#### Nori Compatibility Layer (BND-005)
- `NoriTokenizer` - Nori-style tokenizer with decompound mode support
- `NoriAnalyzer` - Analyzer wrapper with stoptags support
- Bidirectional POS tag mapping (MeCab to Nori and vice versa)
- `DecompoundMode` enum (None/Discard/Mixed)
- `NoriToken` and `WordType` data structures
- Character offset calculation (byte to character conversion)

#### Performance Benchmarks (QA-001)
- Criterion-based benchmark framework
- Trie search benchmarks (exact match, common prefix search)
- Matrix lookup benchmarks (single, batch, Viterbi pattern)
- Memory efficiency measurements
- Cache locality tests
- HTML report generation

#### CI/CD Pipeline (QA-004)
- Comprehensive GitHub Actions workflows
- CI workflow: test suite (3 OS x 3 Rust versions), clippy, rustfmt, coverage
- Release workflow: automated GitHub Releases, multi-platform binary builds
- Documentation workflow: rustdoc + mdBook, GitHub Pages deployment
- Code quality workflow: static analysis, dependency audit, complexity analysis
- Performance benchmark workflow with PR comparison
- Scheduled tasks: daily security audit, weekly dependency checks
- Dependabot integration for automated dependency updates
- PR template for contribution guidelines

#### Build and Deployment
- Workspace configuration with multiple crates
- Release profile optimizations (LTO, single codegen unit, strip)
- Cross-platform build support
- crates.io publishing automation

#### Security
- Security scanning integration
- E2E tests for comprehensive validation
- Dependency vulnerability auditing (cargo-audit, cargo-deny)
- License compliance checking

### Changed

- Updated `rkyv` to 0.8.13+ to fix UB in Arc/Rc impls on OOM (RUSTSEC-2026-0001)
- Updated `pyo3` to 0.24.1+ to fix buffer overflow in PyString::from_object (RUSTSEC-2025-0020)
- Added note about `bincode` 1.3 being unmaintained (RUSTSEC-2025-0141) with migration planned

### Fixed

- Phase 6 example compilation errors
- Core Tokenizer integration and API compatibility issues

### Documentation

- Comprehensive project documentation in docs/book/
- API reference documentation for Rust, Python, Node.js, and WASM
- CLI usage guide
- User dictionary guide
- FAQ and troubleshooting
- Architecture documentation
- Build process documentation
- Dictionary format specification (v2 and v3)
- POS tag mapping reference
- Performance tuning guide
- Migration guide from legacy C/C++ implementation

---

## Legacy Versions (C/C++ Implementation)

The following versions refer to the original C/C++ MeCab-Ko implementation in `/legacy/`.

### [0.9.2] (mecab-0.996)

#### Fixed
- Changed dicdir value in mecabrc to `@prefix@/lib/mecab/dic/mecab-ko-dic`
- Fixed Java SWIG memory leak issue

### [0.9.1] (mecab-0.996)

#### Fixed
- Fixed bug where adding new dictionary entries caused errors
- Fixed automatic left/right context ID lookup and cost calculation

### [0.9.0] (mecab-0.996)

#### Added
- Based on MeCab 0.996
- Added feature to increase connection cost for parts of speech containing left whitespace in dictionary settings (dicrc)

#### Configuration Example
```text
# Configuration to increase connection cost for parts of speech containing left whitespace
# This is a mecab-ko specific setting with the following format:
# <posid 1>,<posid 1 penalty cost>,<posid 2>,<posid 2 penalty cost> ...
#
# Example: 120,6000 => For posid 120 (particle), if left whitespace exists,
# increase connection cost by 6000
left-space-penalty-factor = 120,6000,184,6000,100,500
```

---

## Roadmap

### Short-term (v0.2.0)
- [ ] Complete binary dictionary loader implementation
- [ ] Viterbi algorithm optimization
- [ ] N-best path search
- [ ] Whitespace penalty implementation

### Mid-term (v0.5.0)
- [ ] mecab-ko-dic v3.0 dictionary
- [ ] Performance benchmarks vs competitors
- [ ] Enhanced unknown word handling

### Long-term (v1.0.0)
- [ ] Elasticsearch plugin production deployment
- [ ] Accuracy target: 95%+ on Sejong corpus
- [ ] OpenSearch compatibility
- [ ] Community release

---

## Version Policy

### Version Numbers
- **MAJOR**: Breaking API changes
- **MINOR**: Backward-compatible feature additions
- **PATCH**: Backward-compatible bug fixes

### Support Policy
- Only the latest version is actively supported
- Security patches provided for previous MINOR versions

---

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.9.2...HEAD
[0.9.2]: https://github.com/hephaex/mecab-ko/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/hephaex/mecab-ko/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.9.0
