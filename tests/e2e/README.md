# MeCab-Ko End-to-End (E2E) Test Suite

This directory contains comprehensive end-to-end tests for all MeCab-Ko bindings and interfaces.

## Overview

The E2E test suite ensures that:
- All bindings (CLI, Python, Node.js, WASM) produce consistent results
- Edge cases are handled correctly across all platforms
- Performance meets requirements
- Memory management is correct
- Thread safety is maintained

## Directory Structure

```
e2e/
├── cli/                    # CLI E2E tests (Bats)
│   ├── test_cli_basic.bats
│   └── test_cli_output_formats.bats
├── python/                 # Python binding tests (pytest)
│   ├── conftest.py
│   ├── requirements.txt
│   ├── test_basic_tokenization.py
│   └── test_user_dict.py
├── nodejs/                 # Node.js binding tests (Vitest)
│   ├── package.json
│   ├── vitest.config.js
│   └── basic.test.js
├── wasm/                   # WASM binding tests (Vitest)
│   ├── package.json
│   └── basic.test.js
├── common/                 # Common utilities
│   └── test_runner.sh
├── fixtures/               # Test data
│   ├── test_sentences.json
│   └── user_dict.csv
└── README.md              # This file
```

## Test Fixtures

### test_sentences.json

Common test sentences used across all bindings:
- Basic Korean sentences
- Edge cases (empty, whitespace, punctuation)
- Mixed Korean/English/numbers
- Honorific speech
- Long sentences
- Performance test cases

### user_dict.csv

User dictionary for testing custom vocabulary:
- IT terms (Python, Docker, Kubernetes)
- Modern vocabulary (카카오톡, 스마트폰)
- Company/product names

## Running Tests

### All Tests

```bash
# Run all E2E tests
./tests/e2e/common/test_runner.sh

# Run without rebuilding
./tests/e2e/common/test_runner.sh --no-build
```

### CLI Tests Only

```bash
# Using test runner
./tests/e2e/common/test_runner.sh --cli-only

# Direct execution
cd tests/e2e/cli
export MECAB_BIN=/path/to/mecab-ko
bats test_cli_basic.bats
bats test_cli_output_formats.bats
```

**Prerequisites:**
- [Bats](https://github.com/bats-core/bats-core) test framework
- Built MeCab-Ko CLI binary

### Python Tests Only

```bash
# Using test runner
./tests/e2e/common/test_runner.sh --python-only

# Direct execution
cd tests/e2e/python
pip install -r requirements.txt
pytest -v
```

**Prerequisites:**
- Python 3.9+
- pytest and dependencies
- Built Python binding (via maturin)

**Install Python binding:**
```bash
cd rust/crates/mecab-ko-python
maturin develop --release
```

### Node.js Tests Only

```bash
# Using test runner
./tests/e2e/common/test_runner.sh --nodejs-only

# Direct execution
cd tests/e2e/nodejs
npm install
npm test
```

**Prerequisites:**
- Node.js 18+
- npm
- Built Node.js binding

**Build Node.js binding:**
```bash
cd rust/crates/mecab-ko-node
npm install
npm run build
```

### WASM Tests Only

```bash
# Using test runner
./tests/e2e/common/test_runner.sh --wasm-only

# Direct execution
cd tests/e2e/wasm
npm install
npm test
```

**Prerequisites:**
- Node.js 18+
- wasm-pack
- Built WASM module

**Build WASM module:**
```bash
cd rust/crates/mecab-ko-wasm
wasm-pack build --target web
```

## Test Categories

### 1. Basic Tokenization

Tests core tokenization functionality:
- Simple Korean sentences
- Verb conjugations
- Question sentences
- Compound nouns
- Mixed language text
- Numbers and dates
- Honorific speech

### 2. Edge Cases

Tests boundary conditions:
- Empty strings
- Whitespace-only input
- Punctuation-only input
- Very long sentences
- Invalid UTF-8 (where applicable)
- Null/undefined inputs

### 3. Output Formats

Tests various output formats (CLI):
- Default format (tab-separated)
- Wakati format (space-separated)
- JSON format
- JSONL format
- Custom format strings

### 4. User Dictionary

Tests custom vocabulary:
- Loading user dictionary
- User dict priority over system dict
- Multiple user dictionaries

### 5. Performance

Benchmarks and performance tests:
- Short text parsing speed
- Long text parsing speed
- Batch processing
- Memory usage
- Concurrent parsing

### 6. Memory Management

Tests for memory leaks:
- Repeated parsing
- Large input handling
- Concurrent operations
- Resource cleanup

### 7. Thread Safety

Tests concurrent access:
- Multiple threads parsing simultaneously
- Shared tagger instances
- No data races

## CI Integration

Tests run automatically on:
- Push to main/develop branches
- Pull requests
- Manual workflow dispatch

### Test Matrix

**CLI Tests:**
- OS: Ubuntu, macOS, Windows
- Rust: stable, 1.75.0

**Python Tests:**
- OS: Ubuntu, macOS, Windows
- Python: 3.9, 3.10, 3.11, 3.12

**Node.js Tests:**
- OS: Ubuntu, macOS, Windows
- Node: 18, 20, 21

**WASM Tests:**
- Ubuntu only (browser-agnostic)

## Writing New Tests

### Adding Test Cases

1. Add test sentence to `fixtures/test_sentences.json`:
```json
{
  "id": "new_test_001",
  "description": "Test description",
  "input": "한글 입력",
  "expected_tokens": [
    {"surface": "한글", "pos": "NNG"},
    {"surface": "입력", "pos": "NNG"}
  ]
}
```

2. Use in tests:
```python
# Python
def test_new_case(mecab_tagger, test_sentences):
    test_case = next(tc for tc in test_sentences["test_cases"]
                     if tc["id"] == "new_test_001")
    result = mecab_tagger.parse(test_case["input"])
    assert result is not None
```

### Adding New Test Files

**Python:**
```python
# tests/e2e/python/test_new_feature.py
import pytest

class TestNewFeature:
    def test_something(self, mecab_tagger):
        result = mecab_tagger.parse("테스트")
        assert result is not None
```

**Bats:**
```bash
#!/usr/bin/env bats
# tests/e2e/cli/test_new_feature.bats

@test "new feature works" {
    echo "테스트" | run "$MECAB_BIN" --new-flag
    [ "$status" -eq 0 ]
}
```

**Node.js/WASM:**
```javascript
import { describe, it, expect } from 'vitest';

describe('New Feature', () => {
  it('should work', () => {
    const result = tagger.parse('테스트');
    expect(result).toBeDefined();
  });
});
```

## Cross-Platform Consistency

All bindings should produce consistent results for the same input. The test suite includes consistency checks to ensure:

1. **Token Consistency**: Same tokens across bindings
2. **POS Tag Consistency**: Same POS tags across bindings
3. **Feature Consistency**: Same features across bindings

### Comparing Outputs

```bash
# Parse with different bindings and compare
echo "나는 학교에 갑니다." | mecab-ko > cli.txt
python -c "import mecab_ko; print(mecab_ko.Tagger().parse('나는 학교에 갑니다.'))" > python.txt
diff cli.txt python.txt
```

## Debugging Failed Tests

### Enable Verbose Output

**Python:**
```bash
pytest -vv --tb=long
```

**Node.js/WASM:**
```bash
npm test -- --reporter=verbose
```

**Bats:**
```bash
bats --tap test_file.bats
```

### Check Build Status

```bash
# Verify binaries exist
ls -la rust/target/debug/mecab-ko
ls -la rust/target/release/mecab-ko

# Verify Python binding
python -c "import mecab_ko; print(mecab_ko.__version__)"

# Verify Node.js binding
node -e "const m = require('mecab-ko-node'); console.log(m)"
```

### Common Issues

1. **Binary not found**: Rebuild with `cargo build --release`
2. **Import errors**: Reinstall binding with `maturin develop`
3. **Bats not found**: Install with `npm install -g bats` or package manager
4. **Tests skipped**: Check prerequisites and build status

## Performance Benchmarks

Expected performance targets:

- **Short text (< 50 chars)**: < 1ms per parse
- **Medium text (50-500 chars)**: < 10ms per parse
- **Long text (> 500 chars)**: < 100ms per parse
- **Batch (1000 sentences)**: < 1s total

Run benchmarks:
```bash
# Python
pytest --benchmark-only

# Node.js
npm run benchmark

# CLI
./tests/e2e/common/benchmark.sh
```

## Contributing

When adding new features:

1. Add test cases to `fixtures/test_sentences.json`
2. Write tests for all bindings (CLI, Python, Node.js, WASM)
3. Ensure tests pass on all platforms
4. Update this README if needed
5. Check CI passes before merging

## Resources

- [Bats Documentation](https://bats-core.readthedocs.io/)
- [pytest Documentation](https://docs.pytest.org/)
- [Vitest Documentation](https://vitest.dev/)
- [MeCab Documentation](https://taku910.github.io/mecab/)
- [Project Documentation](../../docs/)

## License

Same as the main project (MIT OR Apache-2.0).
