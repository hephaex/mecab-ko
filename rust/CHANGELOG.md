# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Full mecab-ko-dic build (816,283 entries)
- Accuracy benchmark with full dictionary (Token 15.2%, F1 0.165)
- Unknown word cost optimization for Korean neologisms

### Changed
- HangulAlphaMix pattern cost: +200 → -100 (favor neologisms)
- ProperNoun/CamelCase stronger preference (-600/-400)
- Plain length penalty threshold: 5→6 chars

## [0.3.0] - 2026-03-03

### Added
- **N-best Viterbi Algorithm**
  - `ImprovedNbestSearcher`: K-best candidate tracking per node
  - `NbestPath`, `NbestResult` structs with improved API
  - Iterator support (`iter()`, `IntoIterator`)

- **Custom Analysis Modes**
  - `AnalysisMode` enum: Full, NounsOnly, VerbsOnly, ContentWordsOnly, etc.
  - `PosFilter`: prefix/exact matching, include/exclude lists
  - `LemmatizationMode`: None, PredicatesOnly, All
  - `AnalyzerConfig`: combined mode, filter, lemmatization settings

- **Lattice Visualization**
  - `LatticeViz`: DOT, HTML, Text, JSON output formats
  - `VizOptions`: cost display, POS tags, optimal path highlighting
  - d3-graphviz interactive viewer for HTML output

- **Tokenization Caching**
  - `TokenCache`: LRU cache with RwLock thread safety
  - `CachingTokenizer<T>`: tokenizer wrapper with caching
  - `CacheStats`: hit/miss tracking

- **Streaming API Improvements**
  - `TokenStream`: VecDeque optimization (O(1) dequeue)
  - `ProgressStreamingTokenizer`: progress callback support
  - `LargeFileProcessor`: buffered streaming for large files
  - Smart sentence boundary chunking

- **Memory Optimization v2**
  - `PosTagInterner`: String interning for POS tags
  - `FeatureCache`: LRU feature string deduplication
  - `MemoryStats`: memory usage tracking
  - `Lattice.memory_usage()`, `Tokenizer.memory_stats()` methods

- **Accuracy Measurement**
  - `mecab evaluate` CLI subcommand
  - Token/Sentence/POS Accuracy, Precision/Recall/F1
  - POS-specific accuracy reports
  - Sample evaluation data (160 sentences)

- **Dictionary Quality Tools**
  - `analyzer.rs`: POS distribution, cost distribution analysis
  - Histogram generation, outlier detection (3σ, IQR)
  - `--analyze`, `--fix` CLI flags for dict-validator
  - JSON/text report formats

- **Unknown Word Handling**
  - `WordPattern` enum (Plain, ProperNoun, CamelCase, etc.)
  - Pattern-based cost adjustment
  - POS tag estimation by pattern
  - Emoji detection

- **Compound Noun Decomposition**
  - Jongseong pattern analysis
  - Suffix/prefix auto-detection (들, 님, 신/구)
  - Min 3 syllables, max 2 split points

- **npm WASM Package**
  - mecab-ko-wasm v0.3.0 on npm
  - bundler, nodejs, web targets
  - ESM exports

- **CI/CD Enhancements**
  - Benchmark regression detection (5%, 10% thresholds)
  - PR comment with comparison tables
  - User dictionary validation job
  - SEO improvements (sitemap, robots.txt, JSON-LD)

### Changed
- All crates bumped to v0.3.0
- 213+ tests passing, 63 Elasticsearch tests
- 47 edge case tests added
- Migration Guide v0.2.0 → v0.3.0

### Performance
- 3x+ improvement over v0.2.0
- Tokenization: 1.84µs (short), 11.49µs (medium), 41.14µs (long)
- Streaming throughput optimized

## [0.2.0] - 2026-03-02

### Added
- **Community Contribution System**
  - CONTRIBUTING.md with guidelines
  - Issue templates (bug, feature, question)
  - PR template

- **Korean Dictionary APIs**
  - National Institute of Korean Language API client
  - Basic Korean Dictionary API client
  - Dictionary data converter

- **Neologism Support**
  - Seed dictionary with 123 neologisms (2018-2024)
  - Auto POS estimation (`estimate_pos()`)
  - CSV duplicate/conflict checking

- **CLI Enhancements**
  - `collect` subcommand for dictionary sync
  - Dictionary sync command
  - REPL mode (`--repl`)

- **GitHub Releases**
  - 4-platform binary distribution
  - Automated release workflow

- **Docker Image**
  - Multi-stage Dockerfile (debian-slim)
  - GHCR deployment (linux/amd64, linux/arm64)

- **Documentation Site**
  - mdBook with tutorials
  - GitHub Pages deployment

### Changed
- Breaking changes documented in BREAKING_CHANGES.md
- Version 0.2.0 across workspace

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

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/hephaex/mecab-ko/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hephaex/mecab-ko/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
