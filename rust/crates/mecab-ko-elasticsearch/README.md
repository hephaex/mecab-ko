# mecab-ko-elasticsearch

Elasticsearch/Lucene Nori 호환 한국어 형태소 분석기

## 개요

`mecab-ko-elasticsearch`는 Apache Lucene의 한국어 분석기 Nori와 호환되는 Elasticsearch 플러그인을 제공합니다. MeCab-Ko 엔진을 기반으로 하며, Nori의 모든 주요 기능을 지원합니다.

## 주요 기능

- **Nori 호환 분석기**: `NoriAnalyzer`, `NoriTokenizer`
- **토큰 필터**:
  - `NoriPartOfSpeechStopFilter` - 품사 기반 필터링
  - `NoriReadingFormFilter` - 읽기(발음) 변환
  - `LowercaseFilter` - 소문자 변환
  - `LengthFilter` - 길이 기반 필터링
- **복합명사 분해**: `none`, `discard`, `mixed` 모드
- **사용자 사전 지원**
- **JNI 바인딩**: Java/Elasticsearch와의 네이티브 통합
- **고성능**: Rust로 작성되어 빠른 처리 속도

## 설치

### Cargo

```toml
[dependencies]
mecab-ko-elasticsearch = "0.1"
```

### JNI 바인딩 활성화

```toml
[dependencies]
mecab-ko-elasticsearch = { version = "0.1", features = ["jni-bindings"] }
```

## 사용법

### Rust API

```rust
use mecab_ko_elasticsearch::analyzer::{NoriAnalyzer, DecompoundMode};
use mecab_ko_elasticsearch::config::AnalyzerConfig;

// 기본 설정으로 분석기 생성
let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;

// 텍스트 분석
let tokens = analyzer.analyze("한국어 형태소 분석기")?;

for token in tokens {
    println!("{}: {} [{}]",
        token.surface,
        token.pos_tag,
        token.reading.unwrap_or_default()
    );
}

// 커스텀 설정
let config = AnalyzerConfig::new()
    .with_decompound_mode(DecompoundMode::Mixed)
    .with_stoptags(vec!["J".to_string(), "E".to_string()])
    .with_output_unknown_unigrams(false);

let analyzer = NoriAnalyzer::new(config)?;
```

### 필터 사용

```rust
use mecab_ko_elasticsearch::filter::{
    NoriPartOfSpeechStopFilter,
    NoriReadingFormFilter,
    CompositeFilter,
    TokenFilter
};

// 품사 필터
let pos_filter = NoriPartOfSpeechStopFilter::new(vec!["J".to_string(), "E".to_string()]);
let filtered = pos_filter.filter(tokens)?;

// 읽기 변환 필터
let reading_filter = NoriReadingFormFilter::new();
let filtered = reading_filter.filter(tokens)?;

// 복합 필터
let mut composite = CompositeFilter::new();
composite.add_filter(Box::new(pos_filter));
composite.add_filter(Box::new(reading_filter));
let filtered = composite.filter(tokens)?;
```

### Elasticsearch 설정

```json
{
  "settings": {
    "analysis": {
      "analyzer": {
        "nori_analyzer": {
          "type": "custom",
          "tokenizer": "nori_tokenizer",
          "filter": ["nori_posfilter", "lowercase"]
        }
      },
      "tokenizer": {
        "nori_tokenizer": {
          "type": "nori_tokenizer",
          "decompound_mode": "mixed",
          "user_dictionary": "userdict_ko.txt"
        }
      },
      "filter": {
        "nori_posfilter": {
          "type": "nori_part_of_speech",
          "stoptags": ["J", "E", "SF"]
        }
      }
    }
  }
}
```

## 복합명사 분해 모드

### None
복합명사를 분해하지 않음
```
"형태소분석기" → ["형태소분석기/NNG"]
```

### Discard
원본은 버리고 분해된 형태소만 출력
```
"형태소분석기" → ["형태소/NNG", "분석/NNG", "기/NNG"]
```

### Mixed
원본과 분해된 형태소 모두 출력
```
"형태소분석기" → ["형태소분석기/NNG", "형태소/NNG", "분석/NNG", "기/NNG"]
```

## 품사 태그

Nori 호환 품사 태그:

- `J` - 조사 (JKS, JKO, JKB, JKV, JKQ, JC, JX 통합)
- `E` - 어미 (EF, EC, ETN, ETM 통합)
- `NNG` - 일반 명사
- `NNP` - 고유 명사
- `VV` - 동사
- `VA` - 형용사
- `MAG` - 일반 부사
- 기타 MeCab-Ko 품사 태그

## JNI 바인딩

Java에서 사용하기:

```java
public class NoriAnalyzer {
    static {
        System.loadLibrary("mecab_ko_elasticsearch");
    }

    public static native long createAnalyzer(String configJson);
    public static native String analyzeText(long handle, String text);
    public static native void destroyAnalyzer(long handle);
    public static native String getVersion();
    public static native boolean validateConfig(String configJson);
}
```

## 성능

벤치마크 실행:

```bash
cargo bench --features jni-bindings
```

일반적인 성능 지표 (짧은 텍스트 기준):
- 분석 속도: ~100K tokens/sec
- 메모리 사용: ~10MB
- 멀티스레드 안전

## 개발

### 빌드

```bash
# 기본 빌드
cargo build

# JNI 바인딩 포함
cargo build --features jni-bindings

# Release 빌드
cargo build --release --features jni-bindings
```

### 테스트

```bash
# 단위 테스트
cargo test

# 통합 테스트
cargo test --test integration_test

# 모든 테스트 (JNI 포함)
cargo test --features jni-bindings
```

### 벤치마크

```bash
cargo bench
```

## 아키텍처

```text
┌─────────────────────────────────────────────┐
│         Elasticsearch Plugin                │
├─────────────────────────────────────────────┤
│  JNI Bindings (Java ↔ Rust)                │
├─────────────────────────────────────────────┤
│  Analysis Pipeline                          │
│  ├─ Analyzer                                │
│  ├─ Tokenizer                               │
│  └─ TokenFilter                             │
├─────────────────────────────────────────────┤
│  Nori Compatibility Layer                   │
│  ├─ NoriTokenizer                           │
│  ├─ NoriPartOfSpeechStopFilter             │
│  └─ NoriReadingFormFilter                   │
├─────────────────────────────────────────────┤
│  MeCab-Ko Core Engine                       │
└─────────────────────────────────────────────┘
```

## 라이선스

MIT OR Apache-2.0

## 기여

기여는 언제나 환영합니다! 이슈나 PR을 자유롭게 제출해주세요.

## 관련 프로젝트

- [mecab-ko-core](../mecab-ko-core) - 핵심 형태소 분석 엔진
- [mecab-ko-dict](../mecab-ko-dict) - 한국어 사전
- [Apache Lucene Nori](https://lucene.apache.org/core/9_0_0/analysis/nori/overview-summary.html) - 참조 구현
