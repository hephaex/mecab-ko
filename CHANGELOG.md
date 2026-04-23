# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1] - 2026-04-23

### Added
- Streaming API: `SentenceReader`, `StreamingTokenizer`, `TokenStream`, `ChunkedTokenIterator`
- Buffer overflow protection: `max_buffer_size` (16MB default) for all streaming components
- `show_dict_info` CLI command with actual file sizes and user dictionary listing
- sejong/ module: 36 smoke tests across 7 submodules (hangul, tag_map, ending_rules, splitter, lexicon, postprocess, corrections)

### Changed
- sejong.rs monolith (9,700 lines) split into 11 submodules
- CI workflows consolidated: 22 → 20 (neologism-sync, validate-domain-dict merged)
- 11 GitHub Actions upgraded to latest versions (docker, gradle, maturin, softprops, etc.)
- Marked 50 placeholder tests with `#[ignore]`, cleaned up 49 dead placeholder tests
- Streaming `drain_sentences()` rewritten from `Vec<char>` to `char_indices()` byte-offset tracking

### Fixed
- Streaming: unbounded buffer growth causing OOM on large inputs without newlines
- Streaming: `pos + 1` multi-byte delimiter panic (e.g., `。` U+3002, 3 bytes) → `char_indices()` safe
- Streaming: decimal number split inconsistency between StreamingTokenizer and SentenceReader
- Docker: `id: build` missing on build-push-action v6 (attestation step failure)
- CI: removed deleted `rustsec/audit-check-action`, stale `RUSTSEC-2024-0436` ignore
- CI: fixed `dict-build.yml` dictionary data fallback, `delete-artifact@v5`
- CI: fixed `elasticsearch-plugin-tests.yml` paths trigger
- CI: fixed `ffi-tests.yml` virtualenv and manifest-path
- Clippy: `unnecessary_sort_by` in hot_reload.rs
- Clippy: `single_match_else` and `redundant_closure` in CLI

### Security
- Updated rustls-webpki 0.103.12 → 0.103.13 (RUSTSEC-2026-0104)

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
