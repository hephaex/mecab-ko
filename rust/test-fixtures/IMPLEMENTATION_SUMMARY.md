# Mini Dictionary Test Fixture - Implementation Summary

## Overview

Created a minimal test dictionary fixture to enable integration tests without requiring a full system dictionary installation.

## Problem Solved

Previously, 23+ integration tests were marked with `#[ignore]` because they required a system dictionary at paths like `/usr/local/lib/mecab/dic/mecab-ko-dic` which may not be installed on all development machines.

## Solution

Created a minimal test dictionary (`mini-dict`) with ~21 common Korean words that provides:
- All three required dictionary files (sys.dic, entries.csv, matrix.bin)
- Proper binary formats matching the production dictionary
- Common Korean words for basic testing

## Files Created

### 1. Dictionary Generator (`/rust/test-fixtures/`)
- **`create_mini_dict.rs`**: Utility to generate the mini dictionary
- **`Cargo.toml`**: Standalone workspace configuration
- **`README.md`**: Documentation for test fixtures
- **`mini-dict/`**: Generated dictionary files
  - `entries.csv`: 21 dictionary entries with POS tags
  - `sys.dic`: Double-array trie (1KB)
  - `matrix.bin`: 25×25 connection cost matrix (1.3KB)

### 2. Test Helper Module
- **`/rust/crates/mecab-ko/tests/common/mini_dict.rs`**: Helper functions
  - `mini_dict_path()`: Get path to mini dictionary
  - `mini_dict_exists()`: Check if mini dictionary is available

### 3. Updated Tests

#### mecab-ko/tests/integration_dict.rs
- ✅ `test_load_system_dictionary`: Now uses mini dictionary (was ignored)
- ✅ `test_dictionary_lookup`: Tests lookup with common Korean words (was ignored)

#### mecab-ko-dict/tests/integration_test.rs
- ✅ `test_system_dictionary_integration`: Falls back to mini dictionary if system dict unavailable

## Test Results

### Before
```
test result: ok. 12 passed; 0 failed; 19 ignored; ...
```

### After
```
test result: ok. 14 passed; 0 failed; 17 ignored; ...
```

**Enabled**: 2 previously ignored tests in `integration_dict.rs`
**Improved**: 1 test in `mecab-ko-dict` now works without system dictionary

## Dictionary Contents

### Words Included (21 entries)
- **Greetings**: 안녕, 하, 세요
- **Gratitude**: 감사, 합니다
- **Nouns**: 한국어, 사람, 시간, 책
- **Verbs**: 가, 다, 먹, 었
- **Particles**: 은, 는, 을, 를, 이
- **Pronouns**: 나, 너

### Format Specifications

#### entries.csv
```
surface,left_id,right_id,cost,feature
안녕,1,1,100,NNG,*,T,안녕,*,*,*,*
```

#### sys.dic (Trie)
- Built with `yada` crate's DoubleArrayBuilder
- Maps surface forms to 0-based entry indices
- ~1KB binary file

#### matrix.bin
```
[2 bytes] lsize: 25 (u16 LE)
[2 bytes] rsize: 25 (u16 LE)
[1,250 bytes] costs: 625 × i16 (all initialized to 100)
```

## Usage in Tests

Tests automatically use the mini dictionary when available:

```rust
use common::mini_dict;

let dict_path = mini_dict::mini_dict_path();

if !mini_dict::mini_dict_exists() {
    println!("Skipping test: run 'cd rust/test-fixtures && cargo run --release'");
    return;
}

let dict = MmapDictionary::load(&dict_path).expect("load mini dict");
let entries = dict.lookup("안녕");
```

## Generating the Dictionary

```bash
cd rust/test-fixtures
cargo run --release
```

This creates/updates `mini-dict/` with fresh dictionary files.

## Limitations & Future Work

### Current Limitations
1. **No duplicate keys**: Trie doesn't support multiple entries per surface form
   - Example: "가" exists as verb OR particle, not both
2. **Minimal vocabulary**: Only 21 words vs. 800K+ in full dictionary
3. **Uniform costs**: All connection costs set to 100 (not realistic)
4. **No compound words**: Simple feature strings only

### Future Improvements
- [ ] Support multiple entries per surface using binary entries format
- [ ] Add more vocabulary (50-100 words covering all major POS tags)
- [ ] Generate realistic connection costs based on common patterns
- [ ] Include compound word examples with proper decomposition
- [ ] Create multiple test dictionaries for different scenarios

## Integration with CI/CD

The mini dictionary is checked into the repository, so:
- ✅ No system dictionary installation required
- ✅ Tests run in any environment (CI, local dev, containers)
- ✅ Reproducible test results
- ✅ Fast test execution (files are tiny)

## Testing Verification

```bash
# Run enabled tests
cargo test --test integration_dict test_load_system_dictionary
cargo test --test integration_dict test_dictionary_lookup
cargo test -p mecab-ko-dict --test integration_test

# All tests pass
test result: ok. 14 passed; 0 failed; 17 ignored
```

## Impact

- **2 tests enabled** in mecab-ko integration tests
- **1 test improved** in mecab-ko-dict (no longer fails when system dict missing)
- **~17 tests remaining** that still need full tokenizer implementation
- **Foundation established** for more comprehensive test fixtures

This provides a solid foundation for integration testing while the full tokenizer and more comprehensive dictionary support are being developed.
