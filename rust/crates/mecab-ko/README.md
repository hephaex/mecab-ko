# MeCab-Ko Integration Tests

This directory contains comprehensive integration tests for the MeCab-Ko Rust implementation.

## Test Structure

```
tests/
├── common/                         # Common test utilities
│   ├── mod.rs                      # Main utilities module
│   └── fixtures.rs                 # Fixture loading and management
├── fixtures/                       # Test data fixtures
│   ├── sample_texts.json           # Sample Korean texts
│   ├── expected_results.json       # Expected analysis results
│   └── edge_cases.json             # Edge case inputs
├── integration_basic.rs            # Basic tokenization tests
├── integration_dict.rs             # Dictionary loading/search tests
├── integration_user_dict.rs        # User dictionary tests
├── integration_nori.rs             # Nori compatibility tests
├── integration_kiwi.rs             # Kiwi compatibility tests
├── integration_performance.rs      # Performance regression tests
└── integration_golden.rs           # Golden test integration
```

## Running Tests

### Run all tests
```bash
cd /home/mare/mecab-ko/rust
cargo test --tests
```

### Run specific test file
```bash
cargo test --test integration_basic
cargo test --test integration_dict
cargo test --test integration_golden
```

### Run with output
```bash
cargo test --tests -- --nocapture
```

### Run ignored tests (manual/slow tests)
```bash
cargo test --tests -- --ignored
```

### Run specific test
```bash
cargo test test_hangul_decomposition_integration
```

## Test Categories

### 1. Basic Tests (`integration_basic.rs`)
Tests fundamental tokenization functionality:
- Basic sentence tokenization
- Morpheme extraction
- POS tagging accuracy
- Edge cases (empty input, single characters, etc.)
- Token position tracking
- Hangul crate integration

**Status**: Implemented (waiting for tokenizer implementation)

### 2. Dictionary Tests (`integration_dict.rs`)
Tests dictionary operations:
- System dictionary loading
- Entry lookup and retrieval
- Connection cost matrix
- Trie-based searching
- Memory-mapped access
- Concurrent access

**Status**: Implemented (waiting for dictionary implementation)

### 3. User Dictionary Tests (`integration_user_dict.rs`)
Tests user dictionary functionality:
- User dictionary creation
- CSV format parsing
- Priority handling
- Persistence (save/load)
- Technical terms and proper names

**Status**: Implemented (waiting for user dictionary implementation)

### 4. Nori Compatibility Tests (`integration_nori.rs`)
Tests Elasticsearch Nori compatibility:
- POS tag mapping (MeCab ↔ Nori)
- Decompound modes (none, discard, mixed)
- Token type classification
- Output format compatibility

**Status**: Implemented (waiting for Nori compatibility layer)

### 5. Kiwi Compatibility Tests (`integration_kiwi.rs`)
Tests Kiwi analyzer compatibility:
- POS tag mapping (MeCab ↔ Kiwi)
- Token format and scoring
- Spacing options
- N-best analysis

**Status**: Implemented (waiting for Kiwi compatibility layer)

### 6. Performance Tests (`integration_performance.rs`)
Tests performance characteristics:
- Tokenization throughput
- Dictionary lookup speed
- Memory usage
- Scaling with input size
- Parallel processing

**Status**: Implemented (waiting for implementation)

### 7. Golden Tests (`integration_golden.rs`)
Integration with golden test set:
- Automatic comparison with expected results
- Test coverage verification
- Regression detection
- Report generation

**Status**: Implemented and ready to use

## Test Fixtures

### sample_texts.json
Sample Korean texts with expected morphological analysis results.
Categories: basic, complex, technical

### expected_results.json
Expected analysis results for common Korean sentences.
Includes noun extraction tests and complex grammatical structures.

### edge_cases.json
Edge case inputs to test robustness:
- Empty strings
- Whitespace
- Special characters
- Mixed scripts
- Numbers and punctuation

## Golden Tests

Golden tests are located in `/home/mare/mecab-ko/tests/golden/`:
- `basic.json` - 50 basic sentence tests
- `nouns.json` - 30 noun extraction tests
- `complex.json` - 20 complex sentence tests

These tests serve as the reference for correct behavior and regression detection.

## Test Coverage

### Generate coverage report
```bash
# Install tarpaulin (once)
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --tests --out Html --output-dir coverage

# View report
open coverage/index.html
```

### Coverage goals
- Line coverage: > 80%
- Branch coverage: > 70%
- Public API coverage: 100%

## CI Integration

Tests are automatically run in CI on:
- Every push to main branches
- All pull requests
- Nightly builds

### CI Test Commands
```bash
# Run all tests
cargo test --all-features --tests

# Run with coverage
cargo tarpaulin --tests --out Xml

# Run benchmarks (on main only)
cargo bench --no-run
```

## Writing New Tests

### 1. Add test to appropriate file
Choose the correct integration test file based on functionality.

### 2. Use common utilities
```rust
mod common;
use common::{load_fixtures, compare_morphs, MorphTestCase};

#[test]
fn test_my_feature() {
    let test_cases = load_fixtures("sample_texts.json")
        .expect("Failed to load fixtures");

    // Your test logic here
}
```

### 3. Mark incomplete tests as ignored
```rust
#[test]
#[ignore = "Requires implementation"]
fn test_future_feature() {
    // TODO: Implement once feature is available
    println!("Feature test (placeholder)");
}
```

### 4. Add performance assertions
```rust
use common::perf;

#[test]
fn test_performance() {
    let result = perf::measure("Operation", 1000, || {
        // Your operation
    });

    println!("{}", result.format());
    perf::assert_performance(&result, 100.0); // Max 100μs
}
```

## Test Maintenance

### Update golden tests (USE WITH CAUTION)
```bash
cargo test test_update_golden_results -- --ignored
```

This will update the expected results in golden test files.
**Only run this after manually verifying that the new results are correct!**

### Generate test report
```bash
cargo test test_generate_report -- --ignored --nocapture
```

This generates a detailed report of golden test results.

## Troubleshooting

### Tests fail with "Failed to load fixtures"
Make sure you're running tests from the workspace root:
```bash
cd /home/mare/mecab-ko/rust
cargo test
```

### Ignored tests don't run
Add `--ignored` flag:
```bash
cargo test -- --ignored
```

### Coverage report is empty
Ensure you're running with `--tests` flag:
```bash
cargo tarpaulin --tests
```

## Future Improvements

- [ ] Add property-based testing with `proptest`
- [ ] Add fuzzing tests
- [ ] Add integration tests with real dictionaries
- [ ] Add benchmarking suite with `criterion`
- [ ] Add stress tests for memory and performance
- [ ] Add cross-platform compatibility tests
- [ ] Add Unicode edge case tests

## References

- [Golden Test Set](../../tests/golden/README.md)
- [Project Plan](../../docs/PROJECT_PLAN.md)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
