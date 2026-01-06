# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-01-05

### Added

- Initial implementation of Node.js bindings for MeCab-Ko
- `Mecab` class with the following methods:
  - `new()` - Create instance with default dictionary
  - `withDict(path)` - Create instance with custom dictionary
  - `tokenize(text)` - Full tokenization with POS tags and positions
  - `morphs(text)` - Extract morpheme surface forms
  - `nouns(text)` - Extract nouns only
  - `pos(text)` - Extract [surface, pos] pairs
- `getVersion()` function to get library version
- Full TypeScript type definitions (.d.ts)
- Cross-platform support:
  - macOS (x64, ARM64)
  - Linux (x64, ARM64, glibc/musl)
  - Windows (x64, ARM64)
- Comprehensive test suite with Vitest
- Documentation and examples
- Zero-copy operations for performance
- Thread-safe implementation

### Technical Details

- Built with napi-rs 2.16
- Rust 1.77+ required
- Node.js 16+ required
- Uses mecab-ko-core for core analysis

[Unreleased]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
