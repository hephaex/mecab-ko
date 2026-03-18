# Nori Plugin Compatibility Guide

> Detailed compatibility reference between MeCab-Ko and Elasticsearch's Nori Korean analyzer plugin

---

## Table of Contents

1. [Overview](#overview)
2. [API Compatibility](#api-compatibility)
3. [POS Tag Mapping](#pos-tag-mapping)
4. [Decompound Mode](#decompound-mode)
5. [Token Filters](#token-filters)
6. [Dictionary Compatibility](#dictionary-compatibility)
7. [Configuration Mapping](#configuration-mapping)
8. [Behavioral Differences](#behavioral-differences)
9. [Migration Checklist](#migration-checklist)
10. [Test Queries](#test-queries)

---

## Overview

MeCab-Ko provides a Nori-compatible layer (`mecab-ko-elasticsearch` crate) that implements the same API as Elasticsearch's official Nori analyzer. Both analyzers are based on the MeCab Korean morphological analysis engine with mecab-ko-dic dictionary.

### Relationship

```
Apache Lucene Nori (Java)
         |
         v
Elasticsearch Nori Plugin
         |
         v (API compatible)
MeCab-Ko Elasticsearch (Rust)
         |
         v
mecab-ko-core (Rust Engine)
```

### Key Compatibility Points

| Feature | Nori | MeCab-Ko | Compatible |
|---------|------|----------|------------|
| Tokenizer API | KoreanTokenizer | NoriTokenizer | Yes |
| Analyzer API | KoreanAnalyzer | NoriAnalyzer | Yes |
| Decompound modes | none/discard/mixed | none/discard/mixed | Yes |
| POS filter | nori_part_of_speech | nori_part_of_speech | Yes |
| Reading filter | nori_readingform | nori_readingform | Yes |
| User dictionary | CSV format | CSV format | Yes |
| Stoptags | J, E (unified) | J, E + detailed tags | Yes |

---

## API Compatibility

### NoriTokenizer

MeCab-Ko's `NoriTokenizer` implements the same interface as Lucene's `KoreanTokenizer`:

```rust
// MeCab-Ko Rust API
use mecab_ko_core::nori_compat::{NoriTokenizer, DecompoundMode};

let mut tokenizer = NoriTokenizer::new(DecompoundMode::Mixed, false)?;
let tokens = tokenizer.tokenize("한국어 형태소 분석기")?;

for token in tokens {
    println!("{}: {} [{}-{}]",
        token.surface,
        token.pos_tag,
        token.start_offset,
        token.end_offset
    );
}
```

```java
// Lucene Nori Java API (reference)
KoreanTokenizer tokenizer = new KoreanTokenizer(
    input,
    userDict,
    DecompoundMode.MIXED,
    false
);
```

### NoriAnalyzer

```rust
// MeCab-Ko
use mecab_ko_core::nori_compat::{NoriAnalyzer, DecompoundMode};

let stoptags = vec!["J".to_string(), "E".to_string()];
let mut analyzer = NoriAnalyzer::new(
    None,                      // user_dictionary
    DecompoundMode::Mixed,     // decompound_mode
    stoptags,                  // stoptags
    false                      // output_unknown_unigrams
)?;

let tokens = analyzer.analyze("안녕하세요")?;
```

```java
// Lucene Nori (reference)
KoreanAnalyzer analyzer = new KoreanAnalyzer(
    userDict,
    DecompoundMode.MIXED,
    new HashSet<>(Arrays.asList("J", "E")),
    false
);
```

### Token Attributes

Both produce tokens with these attributes:

| Attribute | Type | Description |
|-----------|------|-------------|
| surface | String | Token text |
| pos_tag | String | POS tag |
| start_offset | int | Character start position |
| end_offset | int | Character end position |
| lemma | String? | Base form (if applicable) |
| reading | String? | Hangul reading (for Hanja) |
| word_type | Enum | KNOWN, UNKNOWN, USER |
| is_decompound | bool | True if from compound decomposition |

---

## POS Tag Mapping

### Nori's Unified Tags

Nori simplifies the Sejong POS tagset by consolidating particles and endings:

```
Nori: J (all particles)
  -> JKS (주격 조사)
  -> JKC (보격 조사)
  -> JKG (관형격 조사)
  -> JKO (목적격 조사)
  -> JKB (부사격 조사)
  -> JKV (호격 조사)
  -> JKQ (인용격 조사)
  -> JX  (보조사)
  -> JC  (접속 조사)

Nori: E (all endings)
  -> EP  (선어말 어미)
  -> EF  (종결 어미)
  -> EC  (연결 어미)
  -> ETN (명사형 전성 어미)
  -> ETM (관형형 전성 어미)
```

### MeCab-Ko's POS Tag Support

MeCab-Ko supports both unified Nori tags and detailed Sejong tags:

```rust
// Using Nori-style unified tags
let stoptags = vec!["J".to_string(), "E".to_string()];

// Using detailed Sejong tags
let stoptags = vec![
    "JKS".to_string(),
    "JKO".to_string(),
    "EF".to_string(),
    "EC".to_string(),
];

// Mixed usage is also supported
let stoptags = vec![
    "J".to_string(),     // All particles
    "EF".to_string(),    // Only final endings
    "SF".to_string(),    // Punctuation
];
```

### Tag Conversion Functions

```rust
use mecab_ko_core::nori_compat::{mecab_to_nori_tag, nori_to_mecab_tag};

// MeCab -> Nori
assert_eq!(mecab_to_nori_tag("JKS"), "J");   // Subject particle -> J
assert_eq!(mecab_to_nori_tag("JKO"), "J");   // Object particle -> J
assert_eq!(mecab_to_nori_tag("EF"), "E");    // Final ending -> E
assert_eq!(mecab_to_nori_tag("NNG"), "NNG"); // Noun stays NNG

// Nori -> MeCab (representative tag)
assert_eq!(nori_to_mecab_tag("J"), "JX");    // J -> 보조사 (representative)
assert_eq!(nori_to_mecab_tag("E"), "EF");    // E -> 종결어미 (representative)
```

### Complete Tag Mapping Table

| Category | Nori Tag | MeCab-Ko Tags | Description |
|----------|----------|---------------|-------------|
| **Nouns** | NNG | NNG | General noun |
| | NNP | NNP | Proper noun |
| | NNB | NNB, NNBC | Dependent noun |
| | NP | NP | Pronoun |
| | NR | NR | Numeral |
| **Verbs** | VV | VV | Verb |
| | VA | VA | Adjective |
| | VX | VX | Auxiliary verb |
| | VCP | VCP | Positive copula |
| | VCN | VCN | Negative copula |
| **Modifiers** | MM | MM | Determiner |
| | MAG | MAG | Adverb |
| | MAJ | MAJ | Conjunctive adverb |
| **Particles** | J | JKS, JKC, JKG, JKO, JKB, JKV, JKQ, JX, JC | All particles (unified) |
| **Endings** | E | EP, EF, EC, ETN, ETM | All endings (unified) |
| **Affixes** | XPN | XPN | Noun prefix |
| | XSN | XSN | Noun suffix |
| | XSV | XSV | Verb suffix |
| | XSA | XSA | Adjective suffix |
| | XR | XR | Root |
| **Symbols** | SF | SF | Terminal punctuation |
| | SP | SP | Comma/colon |
| | SS | SSO, SSC | Brackets |
| | SE | SE | Ellipsis |
| | SO, SY | SO, SY | Other symbols |
| | SL | SL | Foreign |
| | SH | SH | Hanja |
| | SN | SN | Number |
| **Unknown** | UNKNOWN | Unknown | Unknown word |

---

## Decompound Mode

### Mode Comparison

Both Nori and MeCab-Ko support the same decompound modes:

#### None Mode
No compound decomposition - compounds are kept as single tokens.

```
Input: "형태소분석기"
Output: ["형태소분석기/NNG"]
```

#### Discard Mode
Only output decomposed tokens, discard the original compound.

```
Input: "형태소분석기"
Output: ["형태소/NNG", "분석기/NNG"]
```

#### Mixed Mode
Output both original compound and decomposed tokens.

```
Input: "형태소분석기"
Output: ["형태소분석기/NNG", "형태소/NNG", "분석기/NNG"]
```

### Configuration

```json
// Elasticsearch/OpenSearch
{
  "tokenizer": {
    "my_tokenizer": {
      "type": "mecab_ko",
      "decompound_mode": "mixed"
    }
  }
}
```

```rust
// Rust API
use mecab_ko_core::nori_compat::DecompoundMode;

let mode = DecompoundMode::parse("mixed").unwrap();
// or
let mode = DecompoundMode::Mixed;
```

### Decompound Algorithm

MeCab-Ko uses a multi-strategy approach for compound decomposition:

1. **Dictionary-based decomposition** (highest priority)
   - Uses predefined compound patterns
   - Example: "형태소분석기" -> ["형태소", "분석기"]

2. **Suffix extraction**
   - Identifies common suffixes (-화, -적, -님, etc.)
   - Example: "현대화" -> ["현대", "화"]

3. **Prefix extraction**
   - Identifies common prefixes (초-, 최-, 신-, etc.)
   - Example: "초고속" -> ["초", "고속"]

4. **Syllable-based heuristics** (fallback)
   - Analyzes Jongseong patterns for natural boundaries
   - Example: "학교운동장" -> ["학교", "운동장"]

---

## Token Filters

### NoriPartOfSpeechStopFilter

Identical functionality in both implementations:

```json
// Elasticsearch configuration
{
  "filter": {
    "nori_posfilter": {
      "type": "nori_part_of_speech",
      "stoptags": ["J", "E", "SF"]
    }
  }
}
```

```rust
// Rust API
use mecab_ko_elasticsearch::filter::NoriPartOfSpeechStopFilter;

let filter = NoriPartOfSpeechStopFilter::new(vec![
    "J".to_string(),
    "E".to_string(),
    "SF".to_string(),
]);

let filtered = filter.filter(tokens)?;
```

### NoriReadingFormFilter

Converts Hanja to Hangul reading:

```json
// Elasticsearch configuration
{
  "filter": {
    "nori_reading": {
      "type": "nori_readingform"
    }
  }
}
```

Example:
```
Input: "韓國" (Hanja)
Output: "한국" (Hangul reading)
```

### Additional Filters in MeCab-Ko

MeCab-Ko provides additional filters not in Nori:

```rust
use mecab_ko_elasticsearch::filter::{
    LowercaseFilter,
    LengthFilter,
    CompositeFilter,
};

// Lowercase filter
let lowercase = LowercaseFilter::new();

// Length filter
let length = LengthFilter::new(2, 100); // min=2, max=100

// Composite filter (chain multiple filters)
let mut composite = CompositeFilter::new();
composite.add_filter(Box::new(pos_filter));
composite.add_filter(Box::new(lowercase));
composite.add_filter(Box::new(length));

let filtered = composite.filter(tokens)?;
```

---

## Dictionary Compatibility

### Dictionary Format

Both use mecab-ko-dic format. MeCab-Ko uses version 2.1.1-20180720.

### User Dictionary Format

Compatible CSV format:

```csv
# Format: surface,cost,POS,reading
# Lines starting with # are comments

# Technical terms
딥러닝,-1000,NNG,딥러닝
머신러닝,-1000,NNG,머신러닝
GPT,-1000,SL,GPT

# Brand names
삼성전자,-1000,NNP,삼성전자
네이버,-1000,NNP,네이버

# Compound words
형태소분석기,-1000,NNG,형태소분석기
```

### User Dictionary Configuration

```json
// Elasticsearch
{
  "tokenizer": {
    "my_tokenizer": {
      "type": "mecab_ko",
      "user_dict_path": "/path/to/user-dict.csv"
    }
  }
}
```

```rust
// Rust API
let tokenizer = NoriTokenizer::with_dict(
    "/path/to/dict",
    DecompoundMode::Mixed,
    false
)?;
```

---

## Configuration Mapping

### Tokenizer Settings

| Nori Setting | MeCab-Ko Setting | Notes |
|--------------|------------------|-------|
| `type: nori_tokenizer` | `type: mecab_ko` | Different type names |
| `decompound_mode` | `decompound_mode` | Identical |
| `user_dictionary` | `user_dict_path` | Different key name |
| `discard_punctuation` | (use POS filter) | Use stoptags instead |

### Analyzer Settings

| Nori Setting | MeCab-Ko Setting | Notes |
|--------------|------------------|-------|
| `type: nori` | (use custom analyzer) | Build with custom tokenizer |

### Filter Settings

| Nori Setting | MeCab-Ko Setting | Notes |
|--------------|------------------|-------|
| `type: nori_part_of_speech` | `type: nori_part_of_speech` | Identical |
| `stoptags` | `stoptags` | Identical (supports J, E unified tags) |
| `type: nori_readingform` | `type: nori_readingform` | Identical |

---

## Behavioral Differences

### Known Differences

1. **Performance characteristics**
   - MeCab-Ko: ~25-33% faster, 90% less memory
   - Built-in LRU caching (not in Nori)
   - Parallel batch processing support

2. **Extended POS tag support**
   - MeCab-Ko supports both unified (J, E) and detailed tags
   - Nori only supports unified tags in stoptags

3. **Compound decomposition**
   - MeCab-Ko uses multi-strategy decomposition
   - Includes dictionary + suffix/prefix + heuristic approaches
   - Results may differ slightly for edge cases

4. **Cold start time**
   - MeCab-Ko: ~100ms
   - Nori: 2-3 seconds

### Output Equivalence

For standard Korean text, output is functionally equivalent:

```
Input: "안녕하세요"

Nori output:
- 안녕/NNG
- 하/XSV
- 세요/EF

MeCab-Ko output:
- 안녕/NNG
- 하/XSV
- 세요/EF
```

---

## Migration Checklist

### Pre-Migration

- [ ] Identify all indices using Nori analyzer
- [ ] Document current analyzer/filter configurations
- [ ] List all user dictionaries in use
- [ ] Review stoptag configurations

### Migration Steps

- [ ] Install mecab-ko-elasticsearch plugin
- [ ] Create parallel index with MeCab-Ko analyzer
- [ ] Run comparison tests with sample queries
- [ ] Reindex data to new index
- [ ] Update aliases
- [ ] Monitor search quality metrics

### Post-Migration

- [ ] Remove Nori plugin (if no longer needed)
- [ ] Clean up old indices
- [ ] Document new configuration

### Rollback Plan

- [ ] Keep old indices for rollback period
- [ ] Document alias switching procedure
- [ ] Set monitoring alerts for search quality degradation

---

## Test Queries

### Basic Analysis Test

```json
// Test tokenization
POST /_analyze
{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "mixed"
  },
  "text": "한국어 형태소 분석기를 테스트합니다"
}

// Expected tokens:
// 한국어, 형태소, 분석기, 테스트 (with J/E filtered)
```

### Compound Decomposition Test

```json
POST /_analyze
{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "mixed"
  },
  "text": "형태소분석기"
}

// Expected: 형태소분석기, 형태소, 분석기
```

### POS Filter Test

```json
POST /_analyze
{
  "tokenizer": {
    "type": "mecab_ko"
  },
  "filter": [
    {
      "type": "nori_part_of_speech",
      "stoptags": ["J", "E"]
    }
  ],
  "text": "저는 학교에 갑니다"
}

// Expected: 저, 학교, 가 (particles and endings removed)
```

### User Dictionary Test

```json
POST /_analyze
{
  "tokenizer": {
    "type": "mecab_ko",
    "user_dict_path": "/config/user-dict.csv"
  },
  "text": "ChatGPT로 코딩합니다"
}

// Expected: ChatGPT (as single token if in user dict)
```

### Search Equivalence Test

```bash
#!/bin/bash
# Compare search results between Nori and MeCab-Ko

QUERIES=(
  "인공지능"
  "형태소 분석"
  "한국어 처리"
  "딥러닝 모델"
)

for query in "${QUERIES[@]}"; do
  echo "Query: $query"

  # Nori index
  nori_result=$(curl -s "localhost:9200/nori_index/_search" \
    -H "Content-Type: application/json" \
    -d "{\"query\":{\"match\":{\"content\":\"$query\"}}}" | jq '.hits.total.value')

  # MeCab-Ko index
  mecab_result=$(curl -s "localhost:9200/mecab_index/_search" \
    -H "Content-Type: application/json" \
    -d "{\"query\":{\"match\":{\"content\":\"$query\"}}}" | jq '.hits.total.value')

  echo "  Nori: $nori_result hits"
  echo "  MeCab-Ko: $mecab_result hits"
done
```

---

## References

### MeCab-Ko Documentation

- [Main README](/Users/mare/Simon/mecab-ko/README.md)
- [POS Tag Mapping](/Users/mare/Simon/mecab-ko/docs/pos-tag-mapping.md)
- [Elasticsearch Integration](/Users/mare/Simon/mecab-ko/docs/integrations/elasticsearch.md)
- [Nori Compat Module](/Users/mare/Simon/mecab-ko/rust/crates/mecab-ko-core/src/nori_compat.md)

### External References

- [Elasticsearch Nori Plugin](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis-nori.html)
- [Apache Lucene Nori](https://lucene.apache.org/core/9_0_0/analysis/nori/overview-summary.html)
- [MeCab-Ko Original](https://bitbucket.org/eunjeon/mecab-ko)
- [Sejong Corpus POS Tags](https://www.korean.go.kr/)

---

*Last updated: 2026-03-18*
*MeCab-Ko version: 0.5.0*
