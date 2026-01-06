# MeCab-Ko Elasticsearch Plugin - Quick Start Guide

This guide will help you get started with the MeCab-Ko Elasticsearch plugin in 5 minutes.

## Prerequisites

- Elasticsearch 8.11.3+ installed
- Java 17+ installed
- Git installed

## Installation Steps

### 1. Clone and Build

```bash
# Clone repository
git clone https://github.com/mecab-ko/mecab-ko.git
cd mecab-ko

# Build native library (Rust)
cd rust
cargo build --release --features jni-bindings

# Build plugin
cd ../elasticsearch-plugin
./gradlew bundlePlugin
```

### 2. Install Plugin

```bash
# Option A: Use install script
./install.sh /path/to/elasticsearch

# Option B: Use elasticsearch-plugin
/path/to/elasticsearch/bin/elasticsearch-plugin install \
  file:///absolute/path/to/mecab-ko-analyzer-0.1.0.zip
```

### 3. Restart Elasticsearch

```bash
# Systemd
sudo systemctl restart elasticsearch

# Direct
/path/to/elasticsearch/bin/elasticsearch
```

### 4. Verify Installation

```bash
# Check plugins
curl -X GET "localhost:9200/_cat/plugins?v"

# Expected output:
# name    component          version
# node-1  mecab-ko-analyzer  0.1.0
```

## Basic Usage

### Test Analyzer

```bash
curl -X POST "localhost:9200/_analyze?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "analyzer": "mecab_ko",
  "text": "한국어 형태소 분석기"
}'
```

Expected output:
```json
{
  "tokens": [
    {"token": "한국어", "type": "NNG", "position": 0},
    {"token": "형태소", "type": "NNG", "position": 1},
    {"token": "분석", "type": "NNG", "position": 2},
    {"token": "기", "type": "NNG", "position": 3}
  ]
}
```

### Create Index

```bash
curl -X PUT "localhost:9200/korean_docs?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "settings": {
    "analysis": {
      "analyzer": {
        "korean": {
          "type": "mecab_ko",
          "decompound_mode": "mixed",
          "stoptags": ["J", "E"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {"type": "text", "analyzer": "korean"},
      "content": {"type": "text", "analyzer": "korean"}
    }
  }
}'
```

### Index Documents

```bash
# Index a document
curl -X POST "localhost:9200/korean_docs/_doc?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "title": "Elasticsearch 한국어 분석",
  "content": "MeCab-Ko를 이용한 형태소 분석 예제입니다."
}'

# Index another
curl -X POST "localhost:9200/korean_docs/_doc?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "title": "검색 엔진 소개",
  "content": "Elasticsearch는 강력한 검색 기능을 제공합니다."
}'
```

### Search

```bash
curl -X GET "localhost:9200/korean_docs/_search?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "query": {
    "match": {
      "content": "검색"
    }
  }
}'
```

## Configuration Examples

### Decompound Modes

#### None (Default)
```bash
curl -X POST "localhost:9200/_analyze?pretty" -H 'Content-Type: application/json' -d'
{
  "tokenizer": {
    "type": "mecab_ko_tokenizer",
    "decompound_mode": "none"
  },
  "text": "형태소분석기"
}'
# Output: ["형태소분석기"]
```

#### Discard
```bash
curl -X POST "localhost:9200/_analyze?pretty" -H 'Content-Type: application/json' -d'
{
  "tokenizer": {
    "type": "mecab_ko_tokenizer",
    "decompound_mode": "discard"
  },
  "text": "형태소분석기"
}'
# Output: ["형태소", "분석", "기"]
```

#### Mixed
```bash
curl -X POST "localhost:9200/_analyze?pretty" -H 'Content-Type: application/json' -d'
{
  "tokenizer": {
    "type": "mecab_ko_tokenizer",
    "decompound_mode": "mixed"
  },
  "text": "형태소분석기"
}'
# Output: ["형태소분석기", "형태소", "분석", "기"]
```

### POS Filtering

```bash
curl -X POST "localhost:9200/_analyze?pretty" -H 'Content-Type: application/json' -d'
{
  "tokenizer": "mecab_ko_tokenizer",
  "filter": [
    {
      "type": "mecab_ko_part_of_speech",
      "stoptags": ["J", "E", "SF"]
    }
  ],
  "text": "나는 학교에 간다."
}'
# Filters out: 는(J), 에(J), 는다(E), .(SF)
```

## Common POS Tags

| Tag | Description | Example |
|-----|-------------|---------|
| NNG | General noun | 학교, 책, 컴퓨터 |
| NNP | Proper noun | 서울, 한국, 홍길동 |
| VV | Verb | 가다, 먹다, 보다 |
| VA | Adjective | 크다, 작다, 예쁘다 |
| J | Josa (postposition) | 은, 는, 이, 가 |
| E | Eomi (verb ending) | ㄴ다, 습니다, 었다 |
| SF | Final punctuation | ., ?, ! |

## User Dictionary

### 1. Create Dictionary File

```bash
# Create in Elasticsearch config directory
cat > $ES_HOME/config/userdict_ko.txt << EOF
# Custom words
스마트폰,-100000
아이폰,-100000
카카오톡,-100000
EOF
```

### 2. Use in Analyzer

```bash
curl -X PUT "localhost:9200/custom_dict?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "settings": {
    "analysis": {
      "analyzer": {
        "custom_korean": {
          "type": "mecab_ko",
          "user_dictionary": "userdict_ko.txt"
        }
      }
    }
  }
}'
```

### 3. Test

```bash
curl -X POST "localhost:9200/custom_dict/_analyze?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "analyzer": "custom_korean",
  "text": "아이폰과 스마트폰"
}'
```

## Nori Compatibility

MeCab-Ko is fully compatible with Elasticsearch Nori plugin. Use Nori names directly:

```bash
curl -X PUT "localhost:9200/nori_test?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "settings": {
    "analysis": {
      "analyzer": {
        "my_analyzer": {
          "type": "nori",
          "decompound_mode": "mixed"
        }
      },
      "tokenizer": {
        "my_tokenizer": {
          "type": "nori_tokenizer"
        }
      }
    }
  }
}'
```

## Troubleshooting

### Plugin Not Found

```bash
# Check plugin directory
ls -la $ES_HOME/plugins/mecab-ko-analyzer/

# Check Elasticsearch logs
tail -f /var/log/elasticsearch/elasticsearch.log
```

### Native Library Error

```bash
# Verify native library exists
ls -la $ES_HOME/plugins/mecab-ko-analyzer/native/

# Rebuild if missing
cd mecab-ko/rust
cargo build --release --features jni-bindings
cp target/release/libmecab_ko_elasticsearch.so \
   $ES_HOME/plugins/mecab-ko-analyzer/native/
```

### Permission Issues

```bash
# Fix permissions
sudo chown -R elasticsearch:elasticsearch \
  $ES_HOME/plugins/mecab-ko-analyzer/
sudo chmod -R 755 $ES_HOME/plugins/mecab-ko-analyzer/
```

## Next Steps

- Read full [README.md](README.md) for detailed documentation
- Check [examples/](examples/) for configuration templates
- Visit [documentation site](https://mecab-ko.github.io/docs/) for advanced usage
- Join [discussions](https://github.com/mecab-ko/mecab-ko/discussions) for support

## Performance Tips

1. **Choose appropriate decompound mode**
   - Use `none` for exact matching
   - Use `mixed` for better recall
   - Use `discard` for minimal index size

2. **Filter unnecessary POS tags**
   - Remove `J`, `E` for content words only
   - Keep `SF`, `SP` if punctuation matters

3. **Use user dictionary**
   - Add domain-specific terms
   - Improve analysis accuracy

4. **Monitor performance**
   ```bash
   # Check analysis performance
   curl -X GET "localhost:9200/_nodes/stats/indices?pretty"
   ```

Happy analyzing! 🚀
