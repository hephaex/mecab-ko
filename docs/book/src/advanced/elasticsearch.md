# Elasticsearch 통합

MeCab-Ko를 Elasticsearch의 형태소 분석기로 사용하는 방법을 소개합니다.

## 개요

Elasticsearch는 검색 엔진으로, 한국어 검색을 위해 형태소 분석기가 필요합니다. MeCab-Ko는 Elasticsearch 플러그인 형태로 통합할 수 있습니다.

## 설치

### 플러그인 설치

```bash
# Elasticsearch 플러그인 설치
bin/elasticsearch-plugin install \
  https://github.com/hephaex/elasticsearch-analysis-mecab-ko/releases/download/v8.11.0/elasticsearch-analysis-mecab-ko-8.11.0.zip
```

버전 호환성:

| Elasticsearch | Plugin Version |
|---------------|----------------|
| 8.11.x | 8.11.0 |
| 8.10.x | 8.10.0 |
| 7.17.x | 7.17.0 |

### 수동 빌드

```bash
# 소스 다운로드
git clone https://github.com/hephaex/elasticsearch-analysis-mecab-ko.git
cd elasticsearch-analysis-mecab-ko

# 빌드
./gradlew clean build

# 설치
bin/elasticsearch-plugin install file:///path/to/plugin.zip
```

### 사전 설치

```bash
# 사전 디렉토리 생성
sudo mkdir -p /usr/share/elasticsearch/config/mecab-ko-dic

# 사전 다운로드 및 압축 해제
wget https://github.com/hephaex/mecab-ko-dic/releases/latest/download/mecab-ko-dic.tar.gz
tar xzf mecab-ko-dic.tar.gz -C /usr/share/elasticsearch/config/mecab-ko-dic
```

## 설정

### Analyzer 정의

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "dict_path": "/usr/share/elasticsearch/config/mecab-ko-dic",
          "user_dict_path": "/usr/share/elasticsearch/config/user-dic.csv"
        }
      },
      "analyzer": {
        "mecab_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer"
        }
      }
    }
  }
}
```

### Tokenizer 옵션

```json
{
  "tokenizer": {
    "mecab_ko_tokenizer": {
      "type": "mecab_ko",
      "dict_path": "/path/to/dict",
      "user_dict_path": "/path/to/user-dict.csv",
      "output_format": "mecab",
      "space_penalty": -1000,
      "compound_noun_min_length": 2,
      "decompound": true,
      "pos_filter": ["NNG", "NNP", "VV", "VA"],
      "max_unk_length": 24
    }
  }
}
```

옵션 설명:

| 옵션 | 기본값 | 설명 |
|------|--------|------|
| `dict_path` | - | 사전 디렉토리 경로 |
| `user_dict_path` | - | 사용자 사전 파일 경로 |
| `output_format` | `mecab` | 출력 포맷 |
| `space_penalty` | `-1000` | 띄어쓰기 패널티 |
| `compound_noun_min_length` | `2` | 복합명사 최소 길이 |
| `decompound` | `false` | 복합명사 분해 |
| `pos_filter` | - | 품사 필터 (배열) |
| `max_unk_length` | `24` | 미등록어 최대 길이 |

## 사용 예제

### 기본 분석

```json
POST /my_index/_analyze
{
  "analyzer": "mecab_analyzer",
  "text": "형태소 분석을 시작합니다"
}
```

응답:

```json
{
  "tokens": [
    {
      "token": "형태소",
      "start_offset": 0,
      "end_offset": 3,
      "type": "NNG",
      "position": 0
    },
    {
      "token": "분석",
      "start_offset": 4,
      "end_offset": 6,
      "type": "NNG",
      "position": 1
    },
    {
      "token": "을",
      "start_offset": 6,
      "end_offset": 7,
      "type": "JKO",
      "position": 2
    },
    {
      "token": "시작",
      "start_offset": 8,
      "end_offset": 10,
      "type": "NNG",
      "position": 3
    },
    {
      "token": "하",
      "start_offset": 10,
      "end_offset": 11,
      "type": "XSV",
      "position": 4
    },
    {
      "token": "ㅂ니다",
      "start_offset": 11,
      "end_offset": 14,
      "type": "EF",
      "position": 5
    }
  ]
}
```

### 품사 필터링

명사만 추출:

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_noun": {
          "type": "mecab_ko",
          "pos_filter": ["NNG", "NNP", "NNB"]
        }
      },
      "analyzer": {
        "noun_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_noun"
        }
      }
    }
  }
}
```

테스트:

```json
POST /my_index/_analyze
{
  "analyzer": "noun_analyzer",
  "text": "저는 오늘 학교에 갑니다"
}
```

결과: ["오늘", "학교"]

### 복합명사 분해

```json
PUT /my_index
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_decompound": {
          "type": "mecab_ko",
          "decompound": true,
          "compound_noun_min_length": 2
        }
      },
      "analyzer": {
        "decompound_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_decompound"
        }
      }
    }
  }
}
```

테스트:

```json
POST /my_index/_analyze
{
  "analyzer": "decompound_analyzer",
  "text": "한국어형태소분석기"
}
```

결과: ["한국어", "한국", "어", "형태소", "형태", "소", "분석기", "분석", "기"]

## 인덱싱 및 검색

### 인덱스 생성

```json
PUT /documents
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko"
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

### 문서 인덱싱

```json
POST /documents/_doc/1
{
  "title": "Elasticsearch 한국어 검색",
  "content": "MeCab-Ko를 사용한 형태소 분석 예제입니다."
}

POST /documents/_doc/2
{
  "title": "형태소 분석기 비교",
  "content": "여러 한국어 형태소 분석기를 비교합니다."
}
```

### 검색

```json
GET /documents/_search
{
  "query": {
    "match": {
      "content": "형태소 분석"
    }
  }
}
```

### 하이라이팅

```json
GET /documents/_search
{
  "query": {
    "match": {
      "content": "형태소"
    }
  },
  "highlight": {
    "fields": {
      "content": {}
    }
  }
}
```

## 고급 설정

### 멀티 필드

다양한 분석기를 동시에 사용:

```json
PUT /multi_field_index
{
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "korean_analyzer",
        "fields": {
          "ngram": {
            "type": "text",
            "analyzer": "ngram_analyzer"
          },
          "keyword": {
            "type": "keyword"
          }
        }
      }
    }
  }
}
```

### 동의어 처리

```json
PUT /synonym_index
{
  "settings": {
    "analysis": {
      "filter": {
        "korean_synonym": {
          "type": "synonym",
          "synonyms": [
            "컴퓨터, PC, 피씨",
            "휴대폰, 핸드폰, 모바일"
          ]
        }
      },
      "analyzer": {
        "korean_synonym_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["lowercase", "korean_synonym"]
        }
      }
    }
  }
}
```

### 사용자 사전

user-dic.csv:

```csv
# 표면형,품사,비용,기본형
딥러닝,NNG,-1000,딥러닝
머신러닝,NNG,-1000,머신러닝
트랜스포머,NNG,-1000,트랜스포머
GPT,SL,-1000,GPT
```

설정:

```json
{
  "tokenizer": {
    "mecab_ko_custom": {
      "type": "mecab_ko",
      "user_dict_path": "/usr/share/elasticsearch/config/user-dic.csv"
    }
  }
}
```

## 성능 최적화

### 인덱스 샤드 설정

```json
PUT /optimized_index
{
  "settings": {
    "number_of_shards": 3,
    "number_of_replicas": 1,
    "refresh_interval": "30s",
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko"
        }
      }
    }
  }
}
```

### 필터 캐싱

```json
GET /documents/_search
{
  "query": {
    "bool": {
      "filter": {
        "term": {
          "status": "published"
        }
      },
      "must": {
        "match": {
          "content": "검색어"
        }
      }
    }
  }
}
```

### 벌크 인덱싱

```bash
curl -X POST "localhost:9200/documents/_bulk" \
  -H "Content-Type: application/x-ndjson" \
  --data-binary @bulk_data.ndjson
```

bulk_data.ndjson:

```json
{"index":{"_id":"1"}}
{"title":"제목1","content":"내용1"}
{"index":{"_id":"2"}}
{"title":"제목2","content":"내용2"}
```

## 모니터링

### 분석기 성능 확인

```bash
GET /_nodes/stats/indices/indexing
```

### 쿼리 성능 프로파일링

```json
GET /documents/_search
{
  "profile": true,
  "query": {
    "match": {
      "content": "검색어"
    }
  }
}
```

## 실전 예제

### 블로그 검색

```json
PUT /blog
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_tokenizer": {
          "type": "mecab_ko",
          "pos_filter": ["NNG", "NNP", "VV", "VA", "SL", "SH"]
        }
      },
      "analyzer": {
        "blog_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_tokenizer",
          "filter": ["lowercase", "trim"]
        }
      }
    }
  },
  "mappings": {
    "properties": {
      "title": {
        "type": "text",
        "analyzer": "blog_analyzer",
        "boost": 2.0
      },
      "content": {
        "type": "text",
        "analyzer": "blog_analyzer"
      },
      "tags": {
        "type": "keyword"
      },
      "published_date": {
        "type": "date"
      }
    }
  }
}
```

검색:

```json
GET /blog/_search
{
  "query": {
    "bool": {
      "should": [
        {
          "match": {
            "title": {
              "query": "형태소 분석",
              "boost": 2
            }
          }
        },
        {
          "match": {
            "content": "형태소 분석"
          }
        }
      ],
      "filter": {
        "range": {
          "published_date": {
            "gte": "2024-01-01"
          }
        }
      }
    }
  },
  "highlight": {
    "fields": {
      "title": {},
      "content": {}
    }
  },
  "sort": [
    "_score",
    {"published_date": "desc"}
  ]
}
```

### 전자상거래 상품 검색

```json
PUT /products
{
  "settings": {
    "analysis": {
      "tokenizer": {
        "mecab_ko_product": {
          "type": "mecab_ko",
          "decompound": true,
          "pos_filter": ["NNG", "NNP", "SL", "SN"]
        }
      },
      "analyzer": {
        "product_analyzer": {
          "type": "custom",
          "tokenizer": "mecab_ko_product",
          "filter": ["lowercase"]
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
          "keyword": {
            "type": "keyword"
          }
        }
      },
      "description": {
        "type": "text",
        "analyzer": "product_analyzer"
      },
      "price": {
        "type": "float"
      },
      "category": {
        "type": "keyword"
      }
    }
  }
}
```

## 문제 해결

### 플러그인 로딩 실패

```bash
# 로그 확인
tail -f /var/log/elasticsearch/elasticsearch.log

# 플러그인 목록 확인
bin/elasticsearch-plugin list
```

### 사전을 찾을 수 없음

```bash
# 사전 경로 확인
ls -la /usr/share/elasticsearch/config/mecab-ko-dic

# 권한 확인
chown -R elasticsearch:elasticsearch /usr/share/elasticsearch/config/mecab-ko-dic
```

### 성능 저하

```json
# 분석기 캐시 통계 확인
GET /_nodes/stats/indices/query_cache
```

## 상세 문서

더 자세한 내용은 다음 문서를 참조하세요:

- **[Elasticsearch / OpenSearch 통합 가이드](../../../integrations/elasticsearch.md)** - 전체 설치 및 설정 가이드
- **[Nori 호환성 가이드](../../../integrations/nori-compatibility.md)** - Nori 플러그인과의 호환성 및 마이그레이션
- **[예제 설정 파일](../../../integrations/examples/)** - 바로 사용 가능한 설정 템플릿

## 참고 자료

- [Elasticsearch 공식 문서](https://www.elastic.co/guide/en/elasticsearch/reference/current/index.html)
- [Analysis 플러그인 개발](https://www.elastic.co/guide/en/elasticsearch/plugins/current/analysis.html)
- [MeCab-Ko CLI](../cli-usage.md)
- [mecab-ko-elasticsearch 크레이트](../../../../rust/crates/mecab-ko-elasticsearch/README.md)
- [성능 최적화 가이드](../../../../rust/crates/mecab-ko-elasticsearch/PERFORMANCE.md)
