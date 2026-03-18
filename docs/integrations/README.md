# MeCab-Ko Integration Guides

This directory contains integration guides for using MeCab-Ko with various platforms and services.

## Available Guides

### [Elasticsearch / OpenSearch Integration](elasticsearch.md)

Comprehensive guide for integrating MeCab-Ko with Elasticsearch and OpenSearch for Korean text search and analysis.

**Topics covered:**
- Plugin installation and configuration
- Analyzer and tokenizer setup
- Token filters (POS filter, reading form filter)
- User dictionary configuration
- Performance optimization
- Migration from Nori

### [Nori Plugin Compatibility](nori-compatibility.md)

Detailed compatibility reference between MeCab-Ko and Elasticsearch's official Nori Korean analyzer plugin.

**Topics covered:**
- API compatibility matrix
- POS tag mapping (Nori unified tags vs Sejong detailed tags)
- Decompound mode comparison
- Behavioral differences
- Migration checklist

## Example Configurations

The [examples/](examples/) directory contains ready-to-use configuration files:

| File | Description |
|------|-------------|
| [elasticsearch-index-settings.json](examples/elasticsearch-index-settings.json) | Complete Elasticsearch index configuration with Korean analyzers |
| [opensearch-index-settings.json](examples/opensearch-index-settings.json) | OpenSearch index configuration with autocomplete support |
| [user-dictionary-template.csv](examples/user-dictionary-template.csv) | User dictionary template with common Korean terms |
| [test-queries.sh](examples/test-queries.sh) | Shell script to test Elasticsearch integration |

## Quick Start

### 1. Install Plugin

```bash
# Elasticsearch 8.x
bin/elasticsearch-plugin install \
  https://github.com/hephaex/elasticsearch-analysis-mecab-ko/releases/download/v8.11.0/elasticsearch-analysis-mecab-ko-8.11.0.zip

# OpenSearch 2.x
bin/opensearch-plugin install \
  https://github.com/hephaex/opensearch-analysis-mecab-ko/releases/download/v2.11.0/opensearch-analysis-mecab-ko-2.11.0.zip
```

### 2. Create Index

```json
PUT /korean_index
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
        "korean": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["lowercase"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "content": {
        "type": "text",
        "analyzer": "korean"
      }
    }
  }
}
```

### 3. Test Analysis

```json
POST /korean_index/_analyze
{
  "analyzer": "korean",
  "text": "한국어 형태소 분석기를 테스트합니다"
}
```

## Requirements

- Elasticsearch 7.17.x / 8.x or OpenSearch 1.x / 2.x
- Java 11+ (for Elasticsearch/OpenSearch)
- mecab-ko-dic dictionary (included with plugin or manually installed)

## Related Documentation

- [Main MeCab-Ko README](/README.md)
- [POS Tag Mapping Guide](/docs/pos-tag-mapping.md)
- [mecab-ko-elasticsearch Crate](/rust/crates/mecab-ko-elasticsearch/README.md)
- [Performance Guide](/rust/crates/mecab-ko-elasticsearch/PERFORMANCE.md)

## Support

For issues related to:
- **Plugin installation**: Check [Troubleshooting](elasticsearch.md#troubleshooting)
- **Migration from Nori**: See [Migration Checklist](nori-compatibility.md#migration-checklist)
- **Configuration questions**: Create an issue on GitHub

---

*MeCab-Ko version: 0.5.0*
*Last updated: 2026-03-18*
