# Elasticsearch / OpenSearch Integration Guide

> Comprehensive guide for integrating MeCab-Ko with Elasticsearch and OpenSearch for Korean text analysis

---

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Usage Examples](#usage-examples)
6. [Advanced Settings](#advanced-settings)
7. [Performance Optimization](#performance-optimization)
8. [Migration from Nori](#migration-from-nori)
9. [Troubleshooting](#troubleshooting)
10. [Reference](#reference)

---

## Overview

MeCab-Ko provides native integration with Elasticsearch and OpenSearch through the `mecab-ko-elasticsearch` crate. This integration offers:

- **Nori API Compatibility**: Drop-in replacement for Elasticsearch's official Nori analyzer
- **High Performance**: ~25-33% faster than Nori with 90% less memory usage
- **LRU Caching**: ~100x speedup for repeated queries
- **Batch Processing**: ~5-8x faster for bulk indexing operations
- **Korean Language Optimization**: Space-penalty tuning, compound noun decomposition, and POS filtering

### Comparison with Nori

| Feature | MeCab-Ko | Nori (Elasticsearch) |
|---------|----------|----------------------|
| Base dictionary | mecab-ko-dic 2.1.1 | mecab-ko-dic |
| Memory footprint | ~5 MB | ~50 MB |
| Cold start time | ~100 ms | 2-3 seconds |
| Short query throughput | ~200K qps | ~150K qps |
| Cache hit throughput | ~10M qps | ~1M qps |
| JVM dependency | No | Yes |

---

## Architecture

```
+-----------------------------------------------+
|              Elasticsearch/OpenSearch         |
+-----------------------------------------------+
|        Analysis Plugin (Java)                 |
+-----------------------------------------------+
|        JNI Bridge Layer                       |
+-----------------------------------------------+
|        mecab-ko-elasticsearch (Rust)          |
|  +------------------------------------------+ |
|  |  NoriAnalyzer                            | |
|  |  - LRU Cache (thread-safe)               | |
|  |  - Batch Processing (Rayon)              | |
|  +------------------------------------------+ |
|  |  NoriTokenizer                           | |
|  |  - Decompound Mode (none/discard/mixed)  | |
|  |  - User Dictionary Support               | |
|  +------------------------------------------+ |
|  |  Token Filters                           | |
|  |  - NoriPartOfSpeechStopFilter            | |
|  |  - NoriReadingFormFilter                 | |
|  |  - LowercaseFilter                       | |
|  +------------------------------------------+ |
+-----------------------------------------------+
|        mecab-ko-core (Core Engine)            |
|        mecab-ko-dict (Dictionary)             |
+-----------------------------------------------+
```

---

## Installation

### Prerequisites

- Elasticsearch 7.17.x / 8.x or OpenSearch 1.x / 2.x
- Linux (x86_64), macOS (x86_64/ARM64), or Windows
- Java 11+ (for Elasticsearch)

### Method 1: Plugin Installation (Recommended)

```bash
# For Elasticsearch 8.11.x
bin/elasticsearch-plugin install \
  https://github.com/hephaex/elasticsearch-analysis-mecab-ko/releases/download/v8.11.0/elasticsearch-analysis-mecab-ko-8.11.0.zip

# For Elasticsearch 7.17.x
bin/elasticsearch-plugin install \
  https://github.com/hephaex/elasticsearch-analysis-mecab-ko/releases/download/v7.17.0/elasticsearch-analysis-mecab-ko-7.17.0.zip

# For OpenSearch 2.x
bin/opensearch-plugin install \
  https://github.com/hephaex/opensearch-analysis-mecab-ko/releases/download/v2.11.0/opensearch-analysis-mecab-ko-2.11.0.zip
```

### Version Compatibility

| Elasticsearch | Plugin Version | OpenSearch | Plugin Version |
|---------------|----------------|------------|----------------|
| 8.11.x | 8.11.0 | 2.11.x | 2.11.0 |
| 8.10.x | 8.10.0 | 2.10.x | 2.10.0 |
| 7.17.x | 7.17.0 | 1.3.x | 1.3.0 |

### Method 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/hephaex/mecab-ko.git
cd mecab-ko/rust

# Build the native library
cargo build --release -p mecab-ko-elasticsearch --features jni-bindings

# Build the plugin
cd ../legacy/elasticsearch-plugin
./gradlew clean build

# Install the plugin
bin/elasticsearch-plugin install file:///path/to/plugin.zip
```

### Dictionary Installation

```bash
# Create dictionary directory
sudo mkdir -p /usr/share/elasticsearch/config/mecab-ko-dic

# Download and extract dictionary
wget https://bitbucket.org/eunjeon/mecab-ko-dic/downloads/mecab-ko-dic-2.1.1-20180720.tar.gz
tar xzf mecab-ko-dic-2.1.1-20180720.tar.gz -C /usr/share/elasticsearch/config/mecab-ko-dic

# Set permissions
chown -R elasticsearch:elasticsearch /usr/share/elasticsearch/config/mecab-ko-dic
```

---

## Configuration

### Basic Analyzer Configuration

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      },
      "analyzer": {
        "korean_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["lowercase"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "korean_analyzer"
      },
      "content": {
        "type": "text",
        "analyzer": "korean_analyzer"
      }
    }
  }
}
```

### Tokenizer Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `dict_path` | string | built-in | Path to dictionary directory |
| `user_dict_path` | string | null | Path to user dictionary CSV file |
| `decompound_mode` | string | `none` | Compound noun handling: `none`, `discard`, `mixed` |
| `output_unknown_unigrams` | boolean | false | Split unknown words into unigrams |
| `space_penalty` | integer | -1000 | Penalty for spaces in lattice |
| `max_unk_length` | integer | 24 | Maximum length of unknown words |

### Decompound Modes

```json
// Example with "형태소분석기" (morphological analyzer)

// mode: none - No decomposition
// Output: ["형태소분석기/NNG"]

// mode: discard - Only decomposed tokens
// Output: ["형태소/NNG", "분석기/NNG"]

// mode: mixed - Both original and decomposed tokens
// Output: ["형태소분석기/NNG", "형태소/NNG", "분석기/NNG"]
```

### Token Filters

#### NoriPartOfSpeechStopFilter

Filter tokens by POS tags:

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "filter": {
        "nori_posfilter": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E", "SF", "SP"]
        }
      },
      "analyzer": {
        "korean_content_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["nori_posfilter", "lowercase"]
        }
      }
    }
  }
}
```

Common stoptags for Korean:

| Tag | Description | Recommendation |
|-----|-------------|----------------|
| `J` | All particles (JKS, JKO, etc.) | Remove for content indexing |
| `E` | All endings (EF, EC, etc.) | Remove for content indexing |
| `SF` | Terminal punctuation (. ? !) | Usually remove |
| `SP` | Comma/colon punctuation | Usually remove |
| `SS` | Quotes/brackets | Usually remove |

#### NoriReadingFormFilter

Convert Hanja to Hangul reading:

```json
{
  "filter": {
    "nori_reading": {
      "type": "nori_readingform"
    }
  }
}
```

### User Dictionary

Create a CSV file with custom vocabulary:

```csv
# user-dict.csv
# Format: surface,cost,POS,lemma
딥러닝,-1000,NNG,딥러닝
머신러닝,-1000,NNG,머신러닝
트랜스포머,-1000,NNG,트랜스포머
GPT,-1000,SL,GPT
ChatGPT,-1000,SL,ChatGPT
```

Configure user dictionary:

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_custom": {
          "type": "mecab_ko",
          "user_dict_path": "/usr/share/elasticsearch/config/user-dict.csv",
          "decompound_mode": "mixed"
        }
      }
    }
  }
}
```

---

## Usage Examples

### Basic Analysis

```json
POST /my_index/_analyze
{
  "analyzer": "korean_analyzer",
  "text": "한국어 형태소 분석을 시작합니다"
}
```

Response:

```json
{
  "tokens": [
    {
      "token": "한국어",
      "start_offset": 0,
      "end_offset": 3,
      "type": "NNG",
      "position": 0
    },
    {
      "token": "형태소",
      "start_offset": 4,
      "end_offset": 7,
      "type": "NNG",
      "position": 1
    },
    {
      "token": "분석",
      "start_offset": 8,
      "end_offset": 10,
      "type": "NNG",
      "position": 2
    },
    {
      "token": "시작",
      "start_offset": 13,
      "end_offset": 15,
      "type": "NNG",
      "position": 3
    }
  ]
}
```

### Content-Only Analyzer (Nouns + Verbs)

```json
PUT /content_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_content": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      },
      "filter": {
        "content_pos_filter": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E", "SF", "SP", "SS", "SE", "SO", "SW"]
        }
      },
      "analyzer": {
        "content_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_content",
          "filter": ["content_pos_filter", "lowercase"]
        }
      }
    }
  }
}
```

### E-commerce Product Search

```json
PUT /products
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_product": {
          "type": "mecab_ko",
          "decompound_mode": "mixed",
          "user_dict_path": "/config/product-dict.csv"
        }
      },
      "filter": {
        "product_pos_filter": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E", "SF", "SP"]
        },
        "product_synonym": {
          "type": "synonym",
          "synonyms": [
            "노트북, 랩탑, 휴대용컴퓨터",
            "휴대폰, 핸드폰, 스마트폰, 모바일",
            "티비, TV, 텔레비전"
          ]
        }
      },
      "analyzer": {
        "product_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_product",
          "filter": ["product_pos_filter", "lowercase", "product_synonym"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "name": {
        "type": "text",
        "analyzer": "product_analyzer",
        "fields": {
          "keyword": { "type": "keyword" }
        }
      },
      "description": {
        "type": "text",
        "analyzer": "product_analyzer"
      },
      "category": { "type": "keyword" },
      "price": { "type": "float" }
    }
  }
}
```

### Blog/Article Search with Multi-field

```json
PUT /articles
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      },
      "filter": {
        "content_filter": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E"]
        }
      },
      "analyzer": {
        "korean_standard": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["content_filter", "lowercase"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "korean_standard",
        "boost": 2.0
      },
      "content": {
        "type": "text",
        "analyzer": "korean_standard"
      },
      "tags": { "type": "keyword" },
      "published_date": { "type": "date" },
      "author": { "type": "keyword" }
    }
  }
}
```

### Search Query Examples

```json
// Simple match query
GET /articles/_search
{
  "query": {
    "match": {
      "content": "형태소 분석"
    }
  }
}

// Multi-match with boosting
GET /articles/_search
{
  "query": {
    "multi_match": {
      "query": "인공지능 머신러닝",
      "fields": ["title^2", "content"],
      "type": "best_fields"
    }
  }
}

// Bool query with filters
GET /articles/_search
{
  "query": {
    "bool": {
      "must": {
        "match": { "content": "딥러닝" }
      },
      "filter": [
        { "term": { "tags": "AI" } },
        { "range": { "published_date": { "gte": "2024-01-01" } } }
      ]
    }
  },
  "highlight": {
    "fields": {
      "title": {},
      "content": {}
    }
  }
}
```

---

## Advanced Settings

### Index Settings for Korean Search

```json
PUT /optimized_korean_index
{
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1,
    "refresh_interval": "30s",
    "index.mapping.total_fields.limit": 2000,
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      },
      "filter": {
        "korean_stop": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E", "SF", "SP"]
        },
        "length_filter": {
          "type": "length",
          "min": 2
        }
      },
      "analyzer": {
        "korean_index_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["korean_stop", "length_filter", "lowercase"]
        },
        "korean_search_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["korean_stop", "lowercase"]
        }
      }
    }
  }
}
```

### Different Analyzers for Index vs Search

```json
{
  "mappings": {
    "properties": {
      "content": {
        "type": "text",
        "analyzer": "korean_index_analyzer",
        "search_analyzer": "korean_search_analyzer"
      }
    }
  }
}
```

---

## Performance Optimization

### Caching Configuration

The plugin uses LRU caching by default (1024 entries). Adjust for your workload:

```yaml
# elasticsearch.yml
mecab_ko:
  cache:
    size: 4096        # Number of cached analyses
    enabled: true     # Enable/disable caching
```

### Batch Indexing

Use bulk API for indexing large datasets:

```bash
curl -X POST "localhost:9200/articles/_bulk" \
  -H "Content-Type: application/x-ndjson" \
  --data-binary @articles.ndjson
```

articles.ndjson:
```json
{"index":{"_id":"1"}}
{"title":"첫 번째 글","content":"한국어 형태소 분석 예제입니다"}
{"index":{"_id":"2"}}
{"title":"두 번째 글","content":"Elasticsearch 검색 엔진 사용법"}
```

### Memory Tuning

```yaml
# elasticsearch.yml
# Increase heap for large dictionaries
# ES_JAVA_OPTS="-Xms4g -Xmx4g"

# mecab-ko specific settings
mecab_ko:
  dictionary:
    preload: true     # Preload dictionary at startup
    mmap: true        # Use memory-mapped files
```

### Query Performance

```json
// Use filter context for non-scoring queries
GET /articles/_search
{
  "query": {
    "bool": {
      "filter": [
        { "term": { "status": "published" } }
      ],
      "must": [
        { "match": { "content": "검색어" } }
      ]
    }
  }
}

// Enable request caching for repeated queries
GET /articles/_search?request_cache=true
{
  "query": { "match": { "content": "자주 검색하는 내용" } }
}
```

---

## Migration from Nori

### Compatibility

MeCab-Ko analyzer is designed as a drop-in replacement for Elasticsearch's Nori plugin:

| Nori Setting | MeCab-Ko Equivalent |
|--------------|---------------------|
| `nori_tokenizer` | `mecab_ko` |
| `decompound_mode` | Same options: `none`, `discard`, `mixed` |
| `user_dictionary` | `user_dict_path` |
| `nori_part_of_speech` | Same filter name and options |
| `nori_readingform` | Same filter name |

### Migration Steps

1. **Install MeCab-Ko plugin** alongside Nori (different type names)

2. **Create new index with MeCab-Ko analyzer**:

```json
PUT /articles_v2
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      },
      "analyzer": {
        "korean_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["lowercase"]
        }
      }
    }
  }
}
```

3. **Reindex data**:

```json
POST /_reindex
{
  "source": { "index": "articles_v1" },
  "dest": { "index": "articles_v2" }
}
```

4. **Update alias**:

```json
POST /_aliases
{
  "actions": [
    { "remove": { "index": "articles_v1", "alias": "articles" } },
    { "add": { "index": "articles_v2", "alias": "articles" } }
  ]
}
```

### POS Tag Differences

MeCab-Ko uses the Sejong corpus POS tagset. Nori consolidates some tags:

| Nori Tag | MeCab-Ko Tags | Description |
|----------|---------------|-------------|
| `J` | `JKS`, `JKO`, `JKG`, `JKB`, `JKV`, `JKQ`, `JX`, `JC` | All particles |
| `E` | `EP`, `EF`, `EC`, `ETN`, `ETM` | All endings |

Both tag systems are supported in stoptag filters.

---

## Troubleshooting

### Plugin Loading Failure

```bash
# Check plugin status
bin/elasticsearch-plugin list

# View logs
tail -f /var/log/elasticsearch/elasticsearch.log

# Common issues:
# 1. Version mismatch - ensure plugin version matches ES version
# 2. Missing native library - check libmecab_ko_elasticsearch.so exists
```

### Dictionary Not Found

```bash
# Verify dictionary path
ls -la /usr/share/elasticsearch/config/mecab-ko-dic

# Check permissions
chown -R elasticsearch:elasticsearch /usr/share/elasticsearch/config/mecab-ko-dic

# Verify in elasticsearch.yml
cat /etc/elasticsearch/elasticsearch.yml | grep mecab
```

### Slow Analysis Performance

```json
// Check analysis time with explain
POST /my_index/_analyze
{
  "analyzer": "korean_analyzer",
  "text": "테스트 문장",
  "explain": true
}

// Profile query performance
GET /articles/_search
{
  "profile": true,
  "query": { "match": { "content": "검색어" } }
}
```

### Memory Issues

```bash
# Monitor memory usage
curl -X GET "localhost:9200/_nodes/stats/jvm?pretty"

# Check native memory
curl -X GET "localhost:9200/_nodes/stats/process?pretty"
```

---

## Reference

### POS Tag Reference

See [POS Tag Mapping Guide](/Users/mare/Simon/mecab-ko/docs/pos-tag-mapping.md) for complete tag reference.

### Common POS Tags

| Tag | Korean | English |
|-----|--------|---------|
| NNG | 일반 명사 | General noun |
| NNP | 고유 명사 | Proper noun |
| NNB | 의존 명사 | Dependent noun |
| VV | 동사 | Verb |
| VA | 형용사 | Adjective |
| MAG | 일반 부사 | Adverb |
| JKS | 주격 조사 | Subject particle |
| JKO | 목적격 조사 | Object particle |
| JX | 보조사 | Auxiliary particle |
| EF | 종결 어미 | Final ending |

### API Reference

- [mecab-ko-elasticsearch README](/Users/mare/Simon/mecab-ko/rust/crates/mecab-ko-elasticsearch/README.md)
- [Performance Guide](/Users/mare/Simon/mecab-ko/rust/crates/mecab-ko-elasticsearch/PERFORMANCE.md)
- [Quick Start](/Users/mare/Simon/mecab-ko/rust/crates/mecab-ko-elasticsearch/QUICK_START.md)

### External Resources

- [Elasticsearch Analysis Plugins](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis.html)
- [OpenSearch Analysis](https://opensearch.org/docs/latest/analyzers/)
- [MeCab-Ko Project](https://bitbucket.org/eunjeon/mecab-ko)
- [Sejong Corpus POS Tags](https://www.korean.go.kr/)

---

*Last updated: 2026-03-18*
*MeCab-Ko version: 0.5.0*
