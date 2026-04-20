# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-04-20

### Added
- Memory optimization via LazyEntries: runtime memory reduced from 150MB to 34MB (-77%)
- Memory profiler with jemalloc-ctl for runtime memory diagnostics
- String interning for further memory efficiency
- Elasticsearch 8.x and OpenSearch 3.x search engine plugins (JNI-based)
- ReadingFormFilter for search plugin reading form analysis
- Domain dictionary pipeline: IT terms (283K entries), news NNP (weekly auto-update), government agencies (503 entries from gazette corpus)
- News NNP pipeline infrastructure with CI validation gate
- CI accuracy gate test using verified evaluation dataset
- Evaluation dataset expansion from 52 to 82 sentences
- Wiki dump support for neologism collection workflow
- Search plugin ecosystem research documentation

### Changed
- Achieved Clippy zero warnings across entire workspace
- Improved dictionary quality through enrichment pipeline

### Fixed
- Unknown word grouping ignoring space boundaries (PR #1)
- UTF-8 panic in collect-unknown example truncation
- JNI symbol alignment with Java package naming conventions
- Docker build: correct MECAB_DICDIR environment variable name
- Docker build: add missing dictionary compilation step

### Security
- Update rustls-webpki to fix CRL matching vulnerability (0.103.10 to 0.103.12)
- Remove unmaintained bincode dependency

## [0.6.0] - 2025-12-01

- See [v0.6.0 release](https://github.com/hephaex/mecab-ko/releases/tag/v0.6.0)
