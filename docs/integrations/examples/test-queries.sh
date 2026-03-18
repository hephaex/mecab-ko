#!/bin/bash
# MeCab-Ko Elasticsearch Integration Test Queries
# Usage: ./test-queries.sh [elasticsearch_host]

ES_HOST="${1:-localhost:9200}"
INDEX_NAME="mecab_ko_test"

echo "=== MeCab-Ko Elasticsearch Test Queries ==="
echo "Host: $ES_HOST"
echo "Index: $INDEX_NAME"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to run query and check result
run_test() {
    local test_name="$1"
    local endpoint="$2"
    local data="$3"

    echo -n "Testing: $test_name... "

    response=$(curl -s -X POST "$ES_HOST/$endpoint" \
        -H "Content-Type: application/json" \
        -d "$data")

    if echo "$response" | grep -q '"error"'; then
        echo -e "${RED}FAILED${NC}"
        echo "$response" | jq '.error.reason' 2>/dev/null || echo "$response"
        return 1
    else
        echo -e "${GREEN}PASSED${NC}"
        return 0
    fi
}

# ============================================
# 1. Create Test Index
# ============================================
echo ""
echo "=== 1. Creating Test Index ==="

curl -s -X DELETE "$ES_HOST/$INDEX_NAME" > /dev/null 2>&1

response=$(curl -s -X PUT "$ES_HOST/$INDEX_NAME" \
    -H "Content-Type: application/json" \
    -d '{
  "settings": {
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
          "stoptags": ["J", "E"]
        }
      },
      "analyzer": {
        "korean_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["korean_stop", "lowercase"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": { "type": "text", "analyzer": "korean_analyzer" },
      "content": { "type": "text", "analyzer": "korean_analyzer" }
    }
  }
}')

if echo "$response" | grep -q '"acknowledged":true'; then
    echo -e "${GREEN}Index created successfully${NC}"
else
    echo -e "${RED}Failed to create index${NC}"
    echo "$response"
    exit 1
fi

# ============================================
# 2. Test Basic Tokenization
# ============================================
echo ""
echo "=== 2. Basic Tokenization Tests ==="

# Test 1: Simple Korean sentence
run_test "Simple sentence tokenization" "_analyze" '{
  "analyzer": "korean_analyzer",
  "text": "안녕하세요"
}'

# Test 2: Compound noun
run_test "Compound noun decomposition" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "mixed"
  },
  "text": "형태소분석기"
}'

# Test 3: Mixed Korean/English
run_test "Mixed Korean/English text" "_analyze" '{
  "analyzer": "korean_analyzer",
  "text": "Python으로 개발합니다"
}'

# Test 4: Long sentence
run_test "Long sentence" "_analyze" '{
  "analyzer": "korean_analyzer",
  "text": "한국어 자연어 처리를 위한 형태소 분석기를 개발하고 있습니다"
}'

# ============================================
# 3. Test POS Filtering
# ============================================
echo ""
echo "=== 3. POS Filtering Tests ==="

# Test POS filter with stoptags
run_test "POS filter (remove particles)" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko"
  },
  "filter": [{
    "type": "nori_part_of_speech",
    "stoptags": ["J", "E"]
  }],
  "text": "저는 학교에 갑니다"
}'

# Test detailed POS tags
run_test "Detailed POS tags (JKS, JKO)" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko"
  },
  "filter": [{
    "type": "nori_part_of_speech",
    "stoptags": ["JKS", "JKO", "EF"]
  }],
  "text": "그가 책을 읽습니다"
}'

# ============================================
# 4. Test Decompound Modes
# ============================================
echo ""
echo "=== 4. Decompound Mode Tests ==="

# None mode
run_test "Decompound mode: none" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "none"
  },
  "text": "형태소분석기"
}'

# Discard mode
run_test "Decompound mode: discard" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "discard"
  },
  "text": "형태소분석기"
}'

# Mixed mode
run_test "Decompound mode: mixed" "_analyze" '{
  "tokenizer": {
    "type": "mecab_ko",
    "decompound_mode": "mixed"
  },
  "text": "형태소분석기"
}'

# ============================================
# 5. Index and Search Test
# ============================================
echo ""
echo "=== 5. Index and Search Tests ==="

# Index test documents
echo "Indexing test documents..."

curl -s -X POST "$ES_HOST/$INDEX_NAME/_doc/1" \
    -H "Content-Type: application/json" \
    -d '{"title": "한국어 형태소 분석", "content": "MeCab-Ko는 Rust로 작성된 한국어 형태소 분석기입니다."}' > /dev/null

curl -s -X POST "$ES_HOST/$INDEX_NAME/_doc/2" \
    -H "Content-Type: application/json" \
    -d '{"title": "Elasticsearch 검색 엔진", "content": "Elasticsearch는 분산 검색 엔진입니다."}' > /dev/null

curl -s -X POST "$ES_HOST/$INDEX_NAME/_doc/3" \
    -H "Content-Type: application/json" \
    -d '{"title": "자연어 처리 기술", "content": "딥러닝을 활용한 자연어 처리 기술이 발전하고 있습니다."}' > /dev/null

# Refresh index
curl -s -X POST "$ES_HOST/$INDEX_NAME/_refresh" > /dev/null
echo -e "${GREEN}Documents indexed${NC}"

# Search tests
run_test "Match query: 형태소" "$INDEX_NAME/_search" '{
  "query": { "match": { "content": "형태소" } }
}'

run_test "Match query: 검색 엔진" "$INDEX_NAME/_search" '{
  "query": { "match": { "content": "검색 엔진" } }
}'

run_test "Multi-match query" "$INDEX_NAME/_search" '{
  "query": {
    "multi_match": {
      "query": "한국어 분석",
      "fields": ["title^2", "content"]
    }
  }
}'

run_test "Bool query with filter" "$INDEX_NAME/_search" '{
  "query": {
    "bool": {
      "must": { "match": { "content": "기술" } },
      "filter": { "exists": { "field": "title" } }
    }
  }
}'

# ============================================
# 6. Cleanup
# ============================================
echo ""
echo "=== 6. Cleanup ==="
echo -n "Deleting test index... "
curl -s -X DELETE "$ES_HOST/$INDEX_NAME" > /dev/null
echo -e "${GREEN}Done${NC}"

echo ""
echo "=== Test Complete ==="
