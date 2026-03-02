# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - v0.3.0

### Added

#### Improved N-best Path Search (mecab-ko-core)
- `ImprovedNbestSearcher` - True K-best Viterbi algorithm maintaining K candidates per node
- `NbestPath` - Path structure with node IDs, total cost, and rank
- `NbestResult` - Result container with iterator support
- K-best forward pass tracking multiple candidates at each position
- Heap-based N-best backward pass extraction
- Compatibility API (`search_pairs()`) for legacy code
- 8 tests passing, benchmarks comparing legacy vs improved

#### User-defined Analysis Modes (mecab-ko-core)
- `AnalysisMode` enum: Full, NounsOnly, VerbsOnly, AdjectivesOnly, PredicatesOnly, ContentWordsOnly, SurfaceOnly, Lemmatized, PosTagsOnly, Custom
- `PosFilter` - Flexible POS filtering with prefix/exact matching, include/exclude lists
- `LemmatizationMode` enum: None, PredicatesOnly, All
- `AnalyzerConfig` - Composable configuration for analysis mode, filter, lemmatization, length limits
- `AnalyzedToken` - Transformed token structure with optional lemma
- Convenience functions: `extract_nouns()`, `extract_verbs()`, `extract_adjectives()`, `extract_content_words()`, `extract_lemmas()`
- 12 tests passing

#### Lattice Visualization Tool (mecab-ko-core)
- `LatticeViz` - Main visualization builder with options
- `VizFormat` enum: Dot, Html, Text, Json
- `VizOptions` - Configurable output (cost, POS, best path highlighting, colors, direction)
- DOT output for Graphviz with node type coloring
- HTML output with d3-graphviz interactive viewer
- Text dump for debugging (nodes, positions, best path)
- JSON output for programmatic access
- Convenience functions: `lattice_to_dot()`, `lattice_to_html()`, `lattice_to_text()`, `lattice_to_json()`
- 6 tests passing

#### Tokenization Caching (mecab-ko-core)
- `TokenCache` - Thread-safe LRU cache with RwLock
- `CacheConfig` - Configurable max entries, key length limit, stats tracking
- `CacheStats` - Hit/miss tracking with hit rate calculation
- `CachedToken` - Cached token data structure
- `CachingTokenizer<T>` - Generic wrapper for any tokenizer
- `get_or_insert()` - Combined lookup/compute/store operation
- Auto-skip caching for texts exceeding key length limit
- 9 tests passing

#### Parallel Tokenization Benchmarks (mecab-ko-benchmarks)
- `bench_sequential_vs_parallel` - Compare sequential vs parallel processing (100/500/1000 batches)
- `bench_parallel_scaling` - Measure scaling with 1/2/4/8 thread pools
- `bench_parallel_chunked` - Test chunk sizes 50/100/200/500
- `bench_parallel_throughput` - Measure parallel texts/sec

### Changed

- Internal code quality improvements
- Additional test coverage

### Documentation

- Sprint 16 progress tracking in PLAN.md and PROGRESS.md

---

## [0.2.0] - 2026-03-02

### Added

#### Dictionary Synchronization (mecab-ko-dict-sync)
- `OpenDictClient` for National Institute of Korean Language (NIKL) API integration
- `DictConverter` for NIKL to MeCab-Ko format conversion
- 30+ POS tag mappings (명사→NNG, 고유명사→NNP, 동사→VV, etc.)
- Frequency-based cost calculation (high=0, medium=500, low=1000)
- `UserEntry::to_csv_line()` for MeCab-Ko compatible CSV output
- Paginated search support (`search_paginated`)

#### CLI Enhancements (mecab-ko-cli)
- `sync` subcommand for dictionary synchronization
  - `--source opendict` for NIKL OpenDict API
  - `--query` search term
  - `--api-key` or `OPENDICT_API_KEY` environment variable
  - `--output` CSV file output
  - `--append` mode for merging with existing files
  - `--max-results` limit
- `--benchmark N` option for performance measurement
- `--stats` option for analysis statistics
- REPL improvements with 7 output format switching

#### User Dictionary Improvements (mecab-ko-dict)
- `validate()` method for entry validation
- `stats()` method for dictionary statistics
- `remove_duplicates()` for duplicate entry cleanup
- `remove_surface()` for entry removal by surface form
- `estimate_pos()` for automatic POS tag estimation
- `check_csv_duplicates()` for CSV validation
- `check_system_conflicts()` for system dictionary conflict detection

#### Community Contribution
- CONTRIBUTING.md with neologism addition guide
- 4 issue templates (word-request, bug-report, analysis-error, feature-request)
- CODE_OF_CONDUCT.md (Contributor Covenant 2.0)
- PR template with dictionary change section

#### Neologism Dictionary
- 123 neologisms (2018-2024) in `data/user-dict/neologisms.csv`
- POS tags and cost information included

### Changed

- Minimum Rust version updated to 1.75
- Node.js 22 support added
- Python 3.13 support added (via maturin)
- `mecab-ko-dict-sync` modules now public (client, config, error, models)

### Fixed

- WASM build with `wasm-opt = false` for bulk memory compatibility
- Clippy warnings in dict-sync crate

### Documentation

- Migration Guide (docs/MIGRATION_GUIDE.md)
- NIKL API survey (docs/research/dictionary/korean-dict-api-survey.md)
- mecab-ko-dic modernization plan (docs/research/dictionary/mecab-ko-dic-modernization.md)

---

## [0.1.1] - 2026-03-01

### Added

- crates.io publication for all core crates
- GitHub Releases automation (v0.1.1 tag)
- Performance regression detection in CI
- mdBook documentation site
- Docker image (GHCR, linux/amd64, linux/arm64)
- Performance dashboard with Chart.js

### Fixed

- Memory optimization with LazyEntries and mmap
- WASM zstd-sys issue (optional feature)

---

## [0.1.0] - 2026-03-01

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

### Known Limitations (v0.1.0)

The following features are incomplete or have placeholder implementations in this release. Full implementations are planned for v0.2.0:

- **User dictionary support**: The user dictionary feature has placeholder tests and requires full implementation for production use. CSV loading and custom entry registration are not yet functional.
- **Compound noun decomposition**: The Nori compatibility layer (`nori_compat.rs`) has incomplete compound noun decomposition logic (TODO at line 253). Decompound modes may not produce expected results for all compound nouns.
- **Kiwi POS tagging export**: The Kiwi POS tag export functionality has placeholder tests only. Full POS tag mapping to Kiwi format is not yet implemented.
- **Dictionary entry loading**: The dictionary loader (`loader.rs`) contains placeholder implementation. Binary dictionary loading from mecab-ko-dic is not fully functional.

These limitations will be addressed in v0.2.0. See the Roadmap section below for planned improvements.

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

### Short-term (v0.3.0) - In Progress
- [x] N-best path search (ImprovedNbestSearcher)
- [x] User-defined analysis modes
- [x] Lattice visualization tool
- [x] Tokenization caching (LRU)
- [x] Parallel tokenization benchmarks
- [ ] PyPI distribution
- [ ] npm distribution
- [ ] Breaking changes documentation
- [ ] Migration guide v0.2.0 → v0.3.0

### Mid-term (v0.5.0)
- [ ] mecab-ko-dic v3.0 dictionary
- [ ] Performance benchmarks vs competitors
- [ ] Enhanced unknown word handling
- [ ] Streaming API improvements
- [ ] Memory optimization (lazy loading)

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

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/hephaex/mecab-ko/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hephaex/mecab-ko/compare/v0.9.2...v0.1.0
[0.9.2]: https://github.com/hephaex/mecab-ko/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/hephaex/mecab-ko/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.9.0
