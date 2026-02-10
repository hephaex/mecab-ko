# MeCab-Ko Test Fixtures

This directory contains utilities and fixtures for testing MeCab-Ko without requiring a full system dictionary installation.

## Mini Dictionary

The `mini-dict` directory contains a minimal test dictionary with common Korean words suitable for integration testing.

### Contents

- **entries.csv**: Dictionary entries with surface forms, IDs, costs, and POS features
- **sys.dic**: Double-array trie for efficient word lookup
- **matrix.bin**: Connection cost matrix for morpheme analysis

### Words Included

The mini dictionary includes ~21 common Korean words:
- Greetings: 안녕, 하, 세요
- Thank you: 감사, 합니다
- Common nouns: 한국어, 사람, 시간, 책
- Verbs: 가, 다, 먹, 었
- Particles: 은, 는, 을, 를, 이, 가
- Pronouns: 나, 너

### Generating the Mini Dictionary

If the mini dictionary needs to be regenerated:

```bash
cd rust/test-fixtures
cargo run --release
```

This will create/update the `mini-dict` directory with fresh dictionary files.

### Running Tests

The mini dictionary is automatically used by integration tests when available:

```bash
cd rust
cargo test --test integration_dict test_dictionary_lookup
cargo test -p mecab-ko-dict --test integration_test
```

## Architecture

The mini dictionary uses the same format as the full MeCab-Ko dictionary:

1. **Trie (sys.dic)**: Built with the `yada` crate, maps surface forms to entry indices
2. **Entries (entries.csv)**: CSV format with columns: surface,left_id,right_id,cost,feature
3. **Matrix (matrix.bin)**: Binary format with 25x25 connection costs

### Binary Formats

#### entries.csv
```
surface,left_id,right_id,cost,feature
안녕,1,1,100,NNG,*,T,안녕,*,*,*,*
감사,4,4,100,NNG,*,F,감사,*,*,*,*
```

#### matrix.bin
```
[2 bytes] lsize (u16 little-endian)
[2 bytes] rsize (u16 little-endian)
[lsize * rsize * 2 bytes] costs (i16 array, little-endian)
```

## Usage in Tests

Tests can use the mini dictionary through the helper module:

```rust
use common::mini_dict;

let dict_path = mini_dict::mini_dict_path();

if !mini_dict::mini_dict_exists() {
    // Skip test or fail with helpful message
    return;
}

let dict = MmapDictionary::load(&dict_path)
    .expect("Failed to load mini dictionary");
```

## Limitations

- **Unique keys only**: The trie doesn't support duplicate surface forms (e.g., "가" as both verb and particle)
- **Minimal vocabulary**: Only includes ~21 words for basic testing
- **Simple costs**: Connection costs are uniform (100) for simplicity
- **No compound analysis**: Feature strings are basic, without compound decomposition

## Future Improvements

- [ ] Support multiple entries per surface form using binary entries format
- [ ] Add more vocabulary categories (adjectives, adverbs, endings)
- [ ] Generate more realistic connection costs
- [ ] Include compound word examples
- [ ] Add user dictionary test fixtures
