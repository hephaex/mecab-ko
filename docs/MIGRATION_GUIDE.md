# Migration Guide

This guide helps you migrate from MeCab-Ko v0.1.x to v0.2.0.

## Overview

MeCab-Ko v0.2.0 includes several improvements and new features. While we've tried to maintain backward compatibility, some changes may require updates to your code.

## Breaking Changes

### 1. `mecab-ko-dict-sync` Module Exports

The `mecab-ko-dict-sync` crate now exports additional modules for external use:

```rust
// v0.1.x - Only converter types exported
use mecab_ko_dict_sync::{ConverterEntry, DictConverter, UserEntry};

// v0.2.0 - Additional modules available
use mecab_ko_dict_sync::client::OpenDictClient;
use mecab_ko_dict_sync::config::OpenDictConfig;
use mecab_ko_dict_sync::models::{DictEntry, DictDetail};
use mecab_ko_dict_sync::error::{SyncError, Result};
```

**Action Required**: None if you only use the converter types. Update imports if you need API client functionality.

### 2. UserDictionary API Changes

The `UserDictionary` type in `mecab-ko-dict` has new methods:

```rust
// v0.2.0 - New methods added
impl UserDictionary {
    pub fn validate(&self) -> ValidationResult { ... }
    pub fn stats(&self) -> DictionaryStats { ... }
    pub fn remove_duplicates(&mut self) -> usize { ... }
    pub fn remove_surface(&mut self, surface: &str) -> bool { ... }
}
```

**Action Required**: None. These are additive changes.

### 3. CLI `sync` Subcommand

A new `sync` subcommand is available for dictionary synchronization:

```bash
# v0.2.0 - New sync command
mecab-ko sync --query "신조어" --api-key YOUR_KEY
mecab-ko sync -q "메타버스" --output neologisms.csv
```

**Action Required**: None. This is an additive feature.

### 4. DecompoundMode Changes

The `DecompoundMode` enum remains the same but behavior may differ for edge cases:

```rust
pub enum DecompoundMode {
    None,    // Keep compound nouns as-is
    Discard, // Discard decompounded parts
    Mixed,   // Keep both original and parts
}
```

**Action Required**: Test your compound noun handling if you use `DecompoundMode::Mixed`.

## New Features

### Dictionary Synchronization

Sync dictionary entries from the National Institute of Korean Language (NIKL) API:

```rust
use mecab_ko_dict_sync::client::OpenDictClient;
use mecab_ko_dict_sync::config::OpenDictConfig;

let config = OpenDictConfig::new("your-api-key");
let client = OpenDictClient::new(config)?;

let entries = client.search("신조어").await?;
```

### Dictionary Converter

Convert NIKL entries to MeCab-Ko format:

```rust
use mecab_ko_dict_sync::{DictConverter, ConverterEntry};

let converter = DictConverter::new();
let entry = ConverterEntry {
    surface: "챗GPT".to_string(),
    pos: "고유명사".to_string(),
    reading: Some("챗지피티".to_string()),
    frequency: Some(1000),
};

let user_entry = converter.convert_entry(&entry)?;
println!("{}", user_entry.to_csv_line());
// Output: 챗GPT,0,0,0,NNP,*,*,*,챗지피티,챗GPT,챗지피티,*
```

### Streaming Tokenization

Process large texts efficiently:

```rust
use mecab_ko_core::StreamingTokenizer;

let tokenizer = StreamingTokenizer::new()?;
for token in tokenizer.tokenize_iter(large_text) {
    println!("{}", token.surface);
}
```

### User Dictionary Validation

Validate dictionary entries before use:

```rust
use mecab_ko_dict::UserDictionary;

let dict = UserDictionary::load("custom.csv")?;
let result = dict.validate();

if !result.is_valid() {
    for error in result.errors {
        eprintln!("Error: {}", error);
    }
}
```

## Deprecated Features

The following features are deprecated and will be removed in v0.3.0:

| Feature | Replacement |
|---------|-------------|
| `Tokenizer::tokenize_raw()` | Use `Tokenizer::tokenize()` |

## Version Compatibility Matrix

| Component | v0.1.x | v0.2.0 |
|-----------|--------|--------|
| Rust | 1.70+ | 1.75+ |
| Python | 3.8-3.12 | 3.8-3.13 |
| Node.js | 18, 20 | 18, 20, 22 |
| WASM | ES2020+ | ES2020+ |

## Getting Help

- [GitHub Issues](https://github.com/hephaex/mecab-ko/issues)
- [Documentation](https://hephaex.github.io/mecab-ko/)
- [API Reference](https://docs.rs/mecab-ko)
