# End-to-End Tests

This directory contains end-to-end tests for the mecab-ko project across multiple binding interfaces.

## Test Suites

### CLI Tests (`cli/`) — 7 tests
Tests for the MeCab command-line interface binary.
- Version flag, help flag, empty input, invalid dicdir
- Wakati mode, tab-separated output, EOS marker
- Uses PASS/FAIL/SKIP tracking with dictionary gating

### Python Bindings (`python/`) — 13 tests
Tests for the Python binding interface (mecab-ko-python, PyO3).
- Import, version format, constructor (default + invalid dicpath)
- `morphs()`, `nouns()`, `pos()`, `parse()`, `wakati()`
- Empty input handling, EOS/tab verification
- Dict-dependent tests auto-skip via `conftest.py` fixture

### Node.js Bindings (`node/`) — 12 tests
Tests for the Node.js binding interface (mecab-ko-node, napi-rs).
- Import, `getVersion()`, `Mecab.withDict()` error
- `tokenize()`, `morphs()`, `nouns()`, `pos()`, `parse()`
- Token shape validation, empty input handling
- Dict-dependent tests gracefully skip when no dictionary

### WASM Bindings (`wasm/`) — 5 tests (scaffold)
Tests for the WASM binding interface (mecab-ko-wasm, wasm-pack).
- Build artifact detection, module import
- Exported function validation (tokenize, morphs, nouns, pos, parse)
- Basic tokenization, empty input handling
- Gracefully skips when wasm-pack build artifacts not present

## Running E2E Tests

### Locally

```bash
# CLI tests (no build needed — uses cargo run)
bash tests/e2e/cli/test_basic.sh

# Python tests (requires maturin develop)
cd rust/crates/mecab-ko-python && maturin develop --release
pytest tests/e2e/python/

# Node.js tests (requires napi build)
cd rust/crates/mecab-ko-node && npm run build
cd tests/e2e/node && npm test

# WASM tests (requires wasm-pack build)
wasm-pack build rust/crates/mecab-ko-wasm --target web
cd tests/e2e/wasm && npm test
```

### CI

E2E tests run via `.github/workflows/e2e-ffi-tests.yml` on push/PR.

## Dictionary Availability

Tests that require a dictionary will automatically skip when none is available.
- CLI tests check for `rust/test-fixtures/mini-dict/sys.dic`
- Python tests skip via `conftest.py` fixture
- Node.js tests skip via `tryCreateMecab()` helper
- WASM tests skip when build artifacts aren't present
