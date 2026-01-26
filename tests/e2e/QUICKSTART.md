# E2E Tests Quick Start Guide

## Installation

### 1. Validate Setup

```bash
cd /home/mare/mecab-ko/tests/e2e
./validate_setup.sh
```

### 2. Install Dependencies

```bash
# All dependencies
make install-deps

# Or individually
make install-cli-deps      # Bats
make install-python-deps   # pytest, etc.
make install-nodejs-deps   # npm packages
```

## Running Tests

### Quick Test (Recommended First Run)

```bash
# Build and run all tests
make test
```

### Individual Test Suites

```bash
# CLI tests only
make test-cli

# Python tests only
make test-python

# Node.js tests only
make test-nodejs

# WASM tests only
make test-wasm
```

### Without Rebuilding

```bash
# Skip build step (faster)
make test-no-build
```

## Common Tasks

### Check Consistency Across Bindings

```bash
make consistency-check
```

Expected output:
```
====================================================================
MeCab-Ko Cross-Platform Consistency Check
====================================================================

Total tests: 10
Consistent: 10
Inconsistent: 0

✅ All tests passed - bindings are consistent!
```

### Run Benchmarks

```bash
make benchmark
```

Expected output:
```
================================
MeCab-Ko E2E Performance Benchmarks
================================

Benchmarking: Short text (100 iterations)
  Total: 45ms
  Average: 0.45ms/iter
  Throughput: 2222 iter/s
```

### Generate Coverage Report

```bash
make coverage
```

## Development Workflow

### 1. Watch Mode (Auto-run on file change)

Requires `entr`:
```bash
# Install entr
brew install entr  # macOS
sudo apt-get install entr  # Ubuntu

# Watch Python tests
make watch-python

# Watch CLI tests
make watch-cli
```

### 2. Debug Single Test

```bash
# CLI
cd cli
bats test_cli_basic.bats -t "CLI tokenizes simple Korean sentence"

# Python
cd python
pytest test_basic_tokenization.py::TestBasicTokenization::test_simple_sentence -v

# Node.js
cd nodejs
npm test -- basic.test.js -t "should tokenize simple sentence"
```

### 3. Quick Debug Commands

```bash
# Test CLI manually
make debug-cli

# Test Python binding manually
make debug-python
```

## Troubleshooting

### "Binary not found" Error

```bash
# Build CLI
cd ../../rust
cargo build --release --bin mecab-ko
```

### "mecab_ko module not found" (Python)

```bash
# Build Python binding
cd ../../rust/crates/mecab-ko-python
pip install maturin
maturin develop --release
```

### "Bats not found" (CLI tests)

```bash
# Ubuntu
sudo apt-get install bats

# macOS
brew install bats-core

# Manual
git clone https://github.com/bats-core/bats-core.git
cd bats-core
./install.sh /usr/local
```

### Tests Skipping

This is normal! Tests skip gracefully when:
- Bindings not built yet
- Features not implemented
- Dependencies missing

Run `make build` to build all bindings.

## What Gets Tested

### Test Categories

1. **Basic Tokenization** (all bindings)
   - Simple Korean sentences
   - Verb conjugations
   - Questions
   - Compound nouns

2. **Edge Cases** (all bindings)
   - Empty strings
   - Whitespace
   - Punctuation
   - Long text

3. **Output Formats** (CLI)
   - Default format
   - Wakati mode
   - JSON
   - JSONL

4. **User Dictionary** (Python)
   - Loading custom vocabulary
   - Dictionary priority

5. **Performance** (all bindings)
   - Speed benchmarks
   - Memory usage
   - Throughput

6. **Concurrency** (Python, Node.js)
   - Thread safety
   - Parallel parsing

## Test Data

All tests use shared fixtures in `fixtures/`:
- `test_sentences.json` - 12 common test cases
- `user_dict.csv` - Custom vocabulary

## CI Integration

Tests run automatically on:
- Push to main/develop
- Pull requests
- Manual workflow dispatch

View results: `.github/workflows/e2e-tests.yml`

## Next Steps

After successful tests:

1. **Review Results**: Check test output for any skipped tests
2. **Check Coverage**: Run `make coverage` to see what's tested
3. **Add Tests**: Add your own test cases to `fixtures/test_sentences.json`
4. **Contribute**: Submit improvements via pull request

## Help

- Full documentation: `README.md`
- Testing guide: `../../docs/E2E_TESTING.md`
- Implementation details: `IMPLEMENTATION_SUMMARY.md`
- Project docs: `../../docs/`

## Quick Commands Reference

```bash
make help              # Show all available commands
make test              # Run all tests
make test-cli          # CLI tests only
make test-python       # Python tests only
make benchmark         # Performance tests
make consistency-check # Cross-platform check
make coverage          # Coverage report
make clean             # Clean artifacts
make build             # Build all bindings
```
