# Migration Guide

This guide helps you migrate between MeCab-Ko versions.

---

## v0.2.0 → v0.3.0

MeCab-Ko v0.3.0 introduces powerful new features for advanced tokenization, analysis customization, and performance optimization.

### Breaking Changes

#### 1. TokenStream Internal Buffer Change

The `TokenStream` now uses `VecDeque` instead of `Vec` for better performance. This is an internal change, but if you were relying on undocumented behavior, note:

```rust
// v0.2.0 - Internal Vec buffer
// v0.3.0 - Internal VecDeque buffer (O(1) dequeue)

// API remains the same
for token in token_stream {
    println!("{}", token.surface);
}
```

**Action Required**: None if using public API only.

#### 2. StreamingTokenizer Module Reorganization

Additional types are now exported from the streaming module:

```rust
// v0.2.0
use mecab_ko_core::StreamingTokenizer;

// v0.3.0 - Additional types available
use mecab_ko_core::{
    StreamingTokenizer,
    TokenStream,
    ProgressStreamingTokenizer,
    StreamingProgress,
    ProgressCallback,
    ChunkedTokenIterator,
};
```

**Action Required**: None. These are additive changes.

### New Features

#### 1. Improved N-best Path Search

True K-best Viterbi algorithm with better accuracy:

```rust
use mecab_ko_core::ImprovedNbestSearcher;

let searcher = ImprovedNbestSearcher::new(&lattice, k);
let results = searcher.search();

for (rank, path) in results.iter().enumerate() {
    println!("Rank {}: cost={}", rank + 1, path.total_cost);
    for node_id in &path.node_ids {
        // process nodes
    }
}
```

#### 2. User-defined Analysis Modes

Flexible tokenization with custom filtering:

```rust
use mecab_ko_core::{AnalysisMode, PosFilter, AnalyzerConfig};

// Built-in modes
let nouns = extract_nouns(&tokens);
let verbs = extract_verbs(&tokens);
let content_words = extract_content_words(&tokens);
let lemmas = extract_lemmas(&tokens);

// Custom configuration
let config = AnalyzerConfig::new()
    .with_mode(AnalysisMode::Custom)
    .with_filter(PosFilter::include(&["NNG", "NNP", "VV"]))
    .with_lemmatization(LemmatizationMode::PredicatesOnly)
    .with_min_length(2);

let analyzed: Vec<AnalyzedToken> = config.analyze(&tokens);
```

Available modes:
- `Full` - All tokens (default)
- `NounsOnly` - Common and proper nouns
- `VerbsOnly` - Verbs only
- `AdjectivesOnly` - Adjectives only
- `PredicatesOnly` - Verbs and adjectives
- `ContentWordsOnly` - Nouns, verbs, adjectives, adverbs
- `SurfaceOnly` - Surface forms only (no POS)
- `Lemmatized` - Lemmatized forms
- `PosTagsOnly` - POS tags only
- `Custom` - User-defined with PosFilter

#### 3. Lattice Visualization Tool

Debug and understand morphological analysis:

```rust
use mecab_ko_core::{LatticeViz, VizFormat, VizOptions};

// Quick visualization
let dot = lattice_to_dot(&lattice);
let html = lattice_to_html(&lattice);
let text = lattice_to_text(&lattice);
let json = lattice_to_json(&lattice);

// Customized output
let options = VizOptions::new()
    .show_cost(true)
    .show_pos(true)
    .highlight_best_path(true)
    .with_colors(true);

let viz = LatticeViz::new(&lattice)
    .with_options(options)
    .to_format(VizFormat::Html);

std::fs::write("lattice.html", viz)?;
```

#### 4. Tokenization Caching

LRU cache for repeated tokenization:

```rust
use mecab_ko_core::{TokenCache, CacheConfig, CachingTokenizer};

// Create cache with config
let config = CacheConfig::new()
    .max_entries(10000)
    .max_key_length(1000)
    .track_stats(true);

let cache = TokenCache::with_config(config);

// Use with any tokenizer
let caching_tokenizer = CachingTokenizer::new(tokenizer, cache);

// Stats tracking
let stats = caching_tokenizer.cache_stats();
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
```

#### 5. Progress-aware Streaming

Track progress for large file processing:

```rust
use mecab_ko_core::{ProgressStreamingTokenizer, StreamingProgress};

let stream = ProgressStreamingTokenizer::new(tokenizer)
    .with_total_bytes(file_size)
    .with_progress_callback(|progress: StreamingProgress| {
        let percent = progress.percent().unwrap_or(0.0);
        println!("Progress: {:.1}%", percent);
        println!("Tokens generated: {}", progress.tokens_generated);
    });

for token in stream.tokenize_iter(text) {
    // process tokens
}
```

#### 6. Large File Processing

Efficient memory-streaming for large files:

```rust
use mecab_ko_core::LargeFileProcessor;

let processor = LargeFileProcessor::new()?
    .with_buffer_size(65536)  // 64KB buffer
    .with_progress_callback(|progress| {
        println!("{}% complete", progress.percent());
    });

// Process file without loading entirely into memory
let tokens = processor.process_file("large_corpus.txt")?;
```

#### 7. Smart Chunking

Split text at natural boundaries:

```rust
use mecab_ko_core::BatchTokenizer;

// Smart chunking respects sentence boundaries
let chunks = BatchTokenizer::split_into_chunks_smart(
    text,
    1000,  // chunk size
    &['.', '!', '?', '。', '\n']  // delimiters
);

// Overlapping chunks for context preservation
let overlapping = BatchTokenizer::split_with_overlap(
    text,
    1000,  // chunk size
    100    // overlap size
);
```

#### 8. npm Package

WebAssembly bindings now available:

```javascript
// Install: npm install mecab-ko-wasm

import { Tokenizer } from 'mecab-ko-wasm';

const tokenizer = await Tokenizer.new();
const tokens = tokenizer.tokenize("한국어 형태소 분석");

tokens.forEach(token => {
    console.log(`${token.surface}: ${token.pos}`);
});
```

### Deprecated Features

| Feature | Status | Replacement |
|---------|--------|-------------|
| `NbestSearcher::search_simple()` | Deprecated | Use `ImprovedNbestSearcher` |

### Performance Improvements

| Operation | v0.2.0 | v0.3.0 | Improvement |
|-----------|--------|--------|-------------|
| TokenStream dequeue | O(n) | O(1) | ~10x faster for large streams |
| Smart chunking | N/A | O(n) | Memory-efficient processing |
| Cache hit | N/A | O(1) | Instant for repeated texts |

### Version Compatibility Matrix

| Component | v0.2.0 | v0.3.0 |
|-----------|--------|--------|
| Rust | 1.75+ | 1.75+ |
| Python | 3.8-3.13 | 3.8-3.13 |
| Node.js | 18, 20, 22 | 18, 20, 22 |
| WASM | ES2020+ | ES2020+ |
| npm | N/A | mecab-ko-wasm@0.3.0 |

### Migration Checklist

- [ ] Update `Cargo.toml` dependencies to v0.3.0
- [ ] Review any `NbestSearcher` usage → consider `ImprovedNbestSearcher`
- [ ] Update npm package if using WASM: `npm update mecab-ko-wasm`
- [ ] Test tokenization with new features
- [ ] Consider adding caching for repeated text processing
- [ ] Update documentation for new analysis modes

---

## v0.1.x → v0.2.0

This section helps you migrate from MeCab-Ko v0.1.x to v0.2.0.

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
