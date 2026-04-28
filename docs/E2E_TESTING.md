# End-to-End Testing Guide

This document describes the comprehensive E2E testing strategy for MeCab-Ko across all bindings and platforms.

## Overview

The E2E test suite ensures:
- **Correctness**: All bindings produce correct morphological analysis
- **Consistency**: Results are consistent across CLI, Python, Node.js, and WASM
- **Performance**: All bindings meet performance requirements
- **Reliability**: Edge cases and error conditions are handled properly
- **Cross-platform**: Tests pass on Linux, macOS, and Windows

## Test Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Fixtures                            │
│  - test_sentences.json (common test cases)                  │
│  - user_dict.csv (user dictionary tests)                    │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
    ┌─────────────┬───────────────┬───────────────┬──────────┐
    │             │               │               │          │
    ▼             ▼               ▼               ▼          ▼
┌────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
│  CLI   │  │  Python  │  │  Node.js │  │   WASM   │  │Consistency│
│ Tests  │  │  Tests   │  │  Tests   │  │  Tests   │  │  Check   │
│ (Bats) │  │ (pytest) │  │ (Vitest) │  │ (Vitest) │  │ (Python) │
└────────┘  └──────────┘  └──────────┘  └──────────┘  └──────────┘
    │             │               │               │          │
    └─────────────┴───────────────┴───────────────┴──────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │   Test Report    │
                    │  - Pass/Fail     │
                    │  - Coverage      │
                    │  - Performance   │
                    └──────────────────┘
```

## Test Categories

### 1. Functional Tests

Test basic tokenization functionality:

```python
# Example: Basic tokenization test
def test_basic_tokenization(tagger):
    result = tagger.parse("나는 학교에 갑니다.")
    assert "나" in result
    assert "학교" in result
```

**Test cases:**
- Simple Korean sentences
- Verb conjugations
- Question sentences
- Compound nouns
- Mixed Korean/English text
- Numbers and dates
- Honorific speech

### 2. Edge Case Tests

Test boundary conditions:

```python
# Example: Edge case test
def test_empty_string(tagger):
    result = tagger.parse("")
    assert result is not None  # Should handle gracefully
```

**Test cases:**
- Empty strings
- Whitespace-only input
- Punctuation-only input
- Very long sentences (100K+ characters)
- Invalid UTF-8 sequences
- Null/undefined inputs

### 3. Output Format Tests

Test various output formats (primarily CLI):

```bash
# Example: JSON output format test
@test "JSON output is valid" {
    echo "나는 학교에 갑니다." | mecab-ko --format json | jq .
    [ "$?" -eq 0 ]
}
```

**Formats:**
- Default (tab-separated)
- Wakati (space-separated)
- JSON
- JSONL
- Custom format strings

### 4. User Dictionary Tests

Test custom vocabulary support:

```python
# Example: User dictionary test
def test_user_dictionary(user_dict_path):
    tagger = Tagger(user_dict=user_dict_path)
    result = tagger.parse("카카오톡으로 메시지를 보냈다.")
    assert "카카오톡" in result
```

### 5. Performance Tests

Benchmark parsing speed:

```python
# Example: Performance test
def test_performance(benchmark, tagger):
    text = "나는 학교에 갑니다."
    result = benchmark(tagger.parse, text)
    # Should complete in < 1ms
```

**Benchmarks:**
- Short text (< 50 chars): < 1ms
- Medium text (50-500 chars): < 10ms
- Long text (> 500 chars): < 100ms
- Batch (1000 sentences): < 1s

### 6. Memory Tests

Test memory management:

```python
# Example: Memory leak test
def test_no_memory_leak(tagger):
    text = "나는 학교에 갑니다."
    for _ in range(10000):
        result = tagger.parse(text)
    # Should not leak memory
```

### 7. Concurrency Tests

Test thread safety:

```python
# Example: Concurrent parsing
def test_concurrent_parsing(tagger):
    with ThreadPoolExecutor(max_workers=4) as executor:
        futures = [
            executor.submit(tagger.parse, "나는 학교에 갑니다.")
            for _ in range(100)
        ]
        results = [f.result() for f in futures]
    assert all(r is not None for r in results)
```

## Test Fixtures

### test_sentences.json

Centralized test cases used across all bindings:

```json
{
  "test_cases": [
    {
      "id": "basic_001",
      "description": "Simple sentence",
      "input": "나는 학교에 갑니다.",
      "expected_tokens": [
        {"surface": "나", "pos": "NP"},
        {"surface": "는", "pos": "JX"},
        ...
      ]
    }
  ]
}
```

**Benefits:**
- Single source of truth for test data
- Easy to add new test cases
- Language-agnostic format
- Can include expected results

### user_dict.csv

User dictionary for testing custom vocabulary:

```csv
카카오톡,0,0,0,NNP,*,*,*
스마트폰,0,0,0,NNG,*,*,*
인공지능,0,0,0,NNG,*,*,*
```

## Running Tests

### Quick Start

```bash
# Run all tests
cd tests/e2e
make test

# Run specific binding tests
make test-cli
make test-python
make test-nodejs
make test-wasm
```

### CLI Tests

```bash
# Install bats
sudo apt-get install bats  # Ubuntu
brew install bats-core     # macOS

# Run tests
cd tests/e2e/cli
bats test_cli_basic.bats
bats test_cli_output_formats.bats
```

### Python Tests

```bash
# Install dependencies
cd tests/e2e/python
pip install -r requirements.txt

# Build Python binding
cd ../../../rust/crates/mecab-ko-python
maturin develop --release

# Run tests
cd ../../../tests/e2e/python
pytest -v
pytest --benchmark-only  # Run only benchmarks
pytest --cov            # With coverage
```

### Node.js Tests

```bash
# Install dependencies
cd tests/e2e/nodejs
npm install

# Build Node.js binding
cd ../../../rust/crates/mecab-ko-node
npm install
npm run build

# Run tests
cd ../../../tests/e2e/nodejs
npm test
npm test -- --reporter=verbose  # Verbose output
```

### WASM Tests

```bash
# Install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Build WASM
cd rust/crates/mecab-ko-wasm
wasm-pack build --target web

# Run tests
cd ../../../tests/e2e/wasm
npm install
npm test
```

## Consistency Checking

The consistency checker ensures all bindings produce identical results:

```bash
cd tests/e2e/common
./consistency_check.py
```

**What it checks:**
- Token count matches across bindings
- Surface forms match
- POS tags match
- Features match

**Output:**
```
====================================================================
MeCab-Ko Cross-Platform Consistency Check
====================================================================

Total tests: 10
Consistent: 8
Inconsistent: 2

Inconsistent Tests:
--------------------------------------------------------------------
[mixed_001] Python은 프로그래밍 언어입니다...
  Reason: Token differences found
    - Position 0:
      CLI: Python/SL
      Python: Python/SL
```

## Performance Benchmarking

Run performance benchmarks:

```bash
cd tests/e2e/common
./benchmark.sh
```

**Output:**
```
================================
MeCab-Ko E2E Performance Benchmarks
================================

CLI Benchmarks

Benchmarking: Short text (100 iterations)
  Total: 45ms
  Iterations: 100
  Average: 0.45ms/iter
  Throughput: 2222 iter/s

Benchmarking: Medium text (100 iterations)
  Total: 234ms
  Iterations: 100
  Average: 2.34ms/iter
  Throughput: 427 iter/s
```

## CI/CD Integration

Tests run automatically in GitHub Actions:

```yaml
# .github/workflows/e2e-tests.yml
- name: Run E2E tests
  run: |
    cd tests/e2e
    make ci
```

**Test Matrix:**
- **OS**: Ubuntu, macOS, Windows
- **Python**: 3.9, 3.10, 3.11, 3.12
- **Node.js**: 18, 20, 21
- **Rust**: stable, 1.80.0

## Writing New Tests

### 1. Add Test Case to Fixtures

```json
{
  "id": "new_feature_001",
  "description": "Test new tokenization feature",
  "input": "테스트 문장",
  "expected_tokens": [...]
}
```

### 2. Write Tests for Each Binding

**Python:**
```python
def test_new_feature(mecab_tagger, test_sentences):
    test_case = next(
        tc for tc in test_sentences["test_cases"]
        if tc["id"] == "new_feature_001"
    )
    result = mecab_tagger.parse(test_case["input"])
    # Assert expected behavior
```

**Bats:**
```bash
@test "new feature works" {
    echo "테스트 문장" | run "$MECAB_BIN" --new-flag
    [ "$status" -eq 0 ]
    [[ "$output" =~ "expected" ]]
}
```

**Node.js:**
```javascript
it('should support new feature', () => {
    const result = tagger.parse('테스트 문장');
    expect(result).toContain('expected');
});
```

### 3. Run Tests

```bash
make test
make consistency-check
```

## Best Practices

### 1. Test Independence

Each test should be independent:

```python
# ✅ Good: Independent test
def test_parse(mecab_tagger):
    result = mecab_tagger.parse("나는 학교에 갑니다.")
    assert result is not None

# ❌ Bad: Depends on global state
global_result = None

def test_parse_setup(mecab_tagger):
    global global_result
    global_result = mecab_tagger.parse("...")

def test_parse_check():
    assert global_result is not None
```

### 2. Clear Assertions

Make assertions explicit and clear:

```python
# ✅ Good: Clear assertion
def test_tokenization(mecab_tagger):
    result = mecab_tagger.parse("나는 학교에 갑니다.")
    tokens = extract_tokens(result)
    assert len(tokens) == 7
    assert tokens[0].surface == "나"
    assert tokens[0].pos == "NP"

# ❌ Bad: Vague assertion
def test_tokenization(mecab_tagger):
    result = mecab_tagger.parse("나는 학교에 갑니다.")
    assert result  # What does this test?
```

### 3. Skip Unavailable Tests

Skip tests gracefully when dependencies are missing:

```python
# Python
def test_feature(mecab_tagger):
    if not hasattr(mecab_tagger, 'new_feature'):
        pytest.skip("Feature not implemented")
    # Test code...

# Bats
@test "feature works" {
    if [ ! -f "$MECAB_BIN" ]; then
        skip "Binary not built"
    fi
    # Test code...
}
```

### 4. Performance Tests

Use proper benchmarking tools:

```python
# ✅ Good: Using pytest-benchmark
def test_performance(benchmark, mecab_tagger):
    text = "나는 학교에 갑니다."
    result = benchmark(mecab_tagger.parse, text)

# ❌ Bad: Manual timing
def test_performance(mecab_tagger):
    import time
    start = time.time()
    mecab_tagger.parse("...")
    elapsed = time.time() - start
    assert elapsed < 1.0  # Flaky!
```

## Troubleshooting

### Tests Failing

1. **Check build status:**
   ```bash
   make build
   ```

2. **Verify binaries exist:**
   ```bash
   ls -la rust/target/release/mecab-ko
   python -c "import mecab_ko"
   ```

3. **Run with verbose output:**
   ```bash
   pytest -vv --tb=long
   bats --tap test_file.bats
   npm test -- --reporter=verbose
   ```

### Consistency Check Failures

1. **Identify which binding differs:**
   ```bash
   ./common/consistency_check.py
   ```

2. **Compare outputs manually:**
   ```bash
   echo "테스트" | mecab-ko > cli.txt
   python -c "import mecab_ko; print(mecab_ko.Tagger().parse('테스트'))" > py.txt
   diff cli.txt py.txt
   ```

3. **Check dictionary versions:**
   - Ensure all bindings use the same dictionary
   - Verify user dictionary is loaded correctly

### Performance Issues

1. **Profile the code:**
   ```bash
   cargo flamegraph --bin mecab-ko
   ```

2. **Check for memory leaks:**
   ```bash
   valgrind --leak-check=full mecab-ko
   ```

3. **Compare with baseline:**
   ```bash
   ./common/benchmark.sh > new_bench.txt
   # Compare with previous results
   ```

## Future Enhancements

- [ ] Fuzzing tests for robustness
- [ ] Stress tests with millions of sentences
- [ ] Regression tests with known issues
- [ ] Visual test reports (HTML)
- [ ] Automatic performance regression detection
- [ ] Cross-binding output comparison in CI
- [ ] Mobile platform tests (iOS, Android)
- [ ] Browser compatibility matrix for WASM

## Resources

- [Bats Documentation](https://bats-core.readthedocs.io/)
- [pytest Documentation](https://docs.pytest.org/)
- [Vitest Documentation](https://vitest.dev/)
- [GitHub Actions](https://docs.github.com/en/actions)
- [Project Documentation](../README.md)

## Contributing

When adding new features:

1. Add test cases to `fixtures/test_sentences.json`
2. Write tests for all bindings
3. Run consistency check
4. Update this documentation
5. Verify CI passes

For questions or issues, please open an issue on GitHub.
