# MeCab-Ko Search Engine Plugins

Elasticsearch 8.x와 OpenSearch 3.x를 위한 MeCab-Ko 한국어 형태소 분석기 플러그인입니다.

## 지원 버전

| 플랫폼 | 버전 | Lucene |
|--------|------|--------|
| Elasticsearch | 8.11+ | Lucene 9 |
| OpenSearch | 3.0+ | Lucene 10 |

## 프로젝트 구조

```
search-plugins/
├── common/                 # 공통 코드 (JNI, 토큰 처리)
├── elasticsearch/          # Elasticsearch 플러그인
├── opensearch/             # OpenSearch 플러그인
└── native/                 # 네이티브 라이브러리 (빌드 시 생성)
```

## 빌드

### 전체 빌드

```bash
./gradlew buildAll
```

### 개별 플러그인 빌드

```bash
# Elasticsearch 플러그인
./gradlew :elasticsearch:bundlePlugin

# OpenSearch 플러그인
./gradlew :opensearch:bundlePlugin
```

### 네이티브 라이브러리 빌드

네이티브 라이브러리는 Rust로 작성되어 있습니다:

```bash
cd ../rust
cargo build --release -p mecab-ko-elasticsearch --features jni-bindings
```

## 설치

### Elasticsearch

```bash
bin/elasticsearch-plugin install file:///path/to/mecab-ko-analyzer-0.7.2.zip
```

### OpenSearch

```bash
bin/opensearch-plugin install file:///path/to/opensearch-analysis-mecab-ko-0.7.2.zip
```

## 사용법

### Analyzer 사용

```json
{
  "settings": {
    "analysis": {
      "analyzer": {
        "korean_analyzer": {
          "type": "mecab_ko",
          "decompound_mode": "mixed",
          "stoptags": ["J", "E"]
        }
      }
    }
  }
}
```

### Tokenizer 사용

```json
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "korean_tokenizer": {
          "type": "mecab_ko_tokenizer",
          "decompound_mode": "discard",
          "user_dictionary": "userdict_ko.txt"
        }
      }
    }
  }
}
```

### 분석 테스트

```bash
curl -X POST "localhost:9200/_analyze" -H 'Content-Type: application/json' -d'
{
  "analyzer": "mecab_ko",
  "text": "한국어 형태소 분석기 테스트입니다."
}'
```

## 설정 옵션

| 옵션 | 설명 | 기본값 |
|------|------|--------|
| `decompound_mode` | 복합어 분해 모드 (none, discard, mixed) | none |
| `user_dictionary` | 사용자 사전 파일 경로 | 없음 |
| `stoptags` | 필터링할 품사 태그 배열 | ["J", "E"] |
| `output_unknown_unigrams` | 미등록어를 문자 단위로 분할 | false |

## Nori 호환성

Nori 분석기와의 호환성을 위해 다음 별칭을 제공합니다:

| MeCab-Ko | Nori 호환 별칭 |
|----------|---------------|
| mecab_ko | nori |
| mecab_ko_tokenizer | nori_tokenizer |
| mecab_ko_part_of_speech | nori_part_of_speech |
| mecab_ko_reading_form | nori_readingform |

## 라이선스

Apache License 2.0
