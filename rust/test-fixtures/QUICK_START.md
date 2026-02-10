# Mini Dictionary Test Fixture - Quick Start

## What Is This?

A minimal test dictionary with 21 common Korean words that enables integration tests to run without installing the full MeCab-Ko system dictionary.

## Quick Start

### 1. Verify Dictionary Exists

```bash
ls -lh rust/test-fixtures/mini-dict/
# Should show: entries.csv, sys.dic, matrix.bin
```

### 2. Run Tests

```bash
cd rust
cargo test --test integration_dict test_load_system_dictionary
cargo test --test integration_dict test_dictionary_lookup
```

Expected output:
```
Found: 안녕 -> NNG,*,T,안녕,*,*,*,*
Found: 감사 -> NNG,*,F,감사,*,*,*,*
...
test result: ok. 14 passed; 0 failed; 17 ignored
```

### 3. Regenerate Dictionary (If Needed)

```bash
cd rust/test-fixtures
cargo run --release
```

## Using in Your Tests

```rust
use common::mini_dict;

#[test]
fn my_test() {
    let dict_path = mini_dict::mini_dict_path();

    if !mini_dict::mini_dict_exists() {
        println!("Skipping: run 'cd rust/test-fixtures && cargo run --release'");
        return;
    }

    let dict = MmapDictionary::load(&dict_path)
        .expect("Failed to load mini dictionary");

    // Use the dictionary
    let entries = dict.lookup("안녕");
    assert!(!entries.is_empty());
}
```

## Dictionary Contents

21 common Korean words:
- Greetings: 안녕, 하, 세요
- Thank you: 감사, 합니다
- Nouns: 한국어, 사람, 시간, 책
- Verbs: 가, 다, 먹, 었
- Particles: 은, 는, 을, 를, 이
- Pronouns: 나, 너

## Files

```
test-fixtures/
├── mini-dict/
│   ├── entries.csv    (733 bytes)  - Dictionary entries
│   ├── sys.dic        (1.0 KB)     - Trie for lookup
│   └── matrix.bin     (1.3 KB)     - Connection costs
├── create_mini_dict.rs              - Generator
└── README.md                        - Full documentation
```

## Troubleshooting

### Tests Say "mini dictionary not found"

Run:
```bash
cd rust/test-fixtures
cargo run --release
```

### Want to Add More Words?

Edit `create_mini_dict.rs` and add to the `entries` array in `create_entries_csv()`.

### Dictionary Files Corrupt?

Regenerate:
```bash
cd rust/test-fixtures
rm -rf mini-dict
cargo run --release
```

## What's Next?

This dictionary enables basic integration tests. As the tokenizer is implemented, more sophisticated tests can use this dictionary as a foundation.

See `README.md` for full documentation and `IMPLEMENTATION_SUMMARY.md` for technical details.
