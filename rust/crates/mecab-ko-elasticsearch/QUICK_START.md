# Quick Start Guide

## 기본 사용법

### 1. 가장 간단한 사용

```rust
use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::DecompoundMode;

// 기본 설정 (캐시 활성화, J/E 필터링)
let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed)?;

// 분석
let tokens = analyzer.analyze("한국어 형태소 분석기")?;

for token in tokens {
    println!("{}: {}", token.surface, token.pos_tag);
}
```

## 사용 시나리오별 최적 설정

### 검색 엔진 (Elasticsearch)

```rust
use mecab_ko_elasticsearch::config::AnalyzerConfig;

let config = AnalyzerConfig::new()
    .with_decompound_mode(DecompoundMode::Mixed)  // 검색 recall 향상
    .with_stoptags(vec!["J".to_string(), "E".to_string()]);  // 불필요한 토큰 제거

let analyzer = NoriAnalyzer::new(config)?;  // 기본 캐시 (1024)
```

**Why?**
- Mixed mode: "형태소분석기" → "형태소분석기", "형태소", "분석", "기" (모두 검색 가능)
- 캐시: 동일 쿼리 반복 시 100배 빠름
- J/E 필터: 인덱스 크기 감소

### 실시간 API (높은 QPS)

```rust
let analyzer = NoriAnalyzer::with_cache_size(config, 4096)?;  // 큰 캐시
```

**Why?**
- 4096 엔트리: 더 높은 캐시 히트율
- 레이턴시 최소화
- 메모리는 조금 더 사용 (~800KB)

### 배치 인덱싱 (대량 문서)

```rust
let analyzer = NoriAnalyzer::without_cache(config)?;  // 캐시 비활성화

#[cfg(feature = "batch")]
{
    let documents = vec!["doc1", "doc2", "doc3", /* ... */];
    let results = analyzer.analyze_batch(&documents)?;
}
```

**Why?**
- 캐시 불필요: 문서는 한 번만 처리
- 병렬 처리: CPU 코어 모두 활용
- 최대 처리량: 5-8배 빠름

### 메모리 제약 환경 (임베디드, IoT)

```rust
let analyzer = NoriAnalyzer::with_cache_size(config, 64)?;  // 작은 캐시
// 또는
let analyzer = NoriAnalyzer::without_cache(config)?;  // 캐시 없음
```

**Why?**
- 최소 메모리 사용
- 여전히 빠른 처리

## 성능 모니터링

### 캐시 통계 확인

```rust
if let Some((capacity, size)) = analyzer.cache_stats() {
    println!("Cache utilization: {}/{}  ({:.1}%)",
             size, capacity, (size as f64 / capacity as f64) * 100.0);
}
```

### 캐시 관리

```rust
// 캐시 초기화 (메모리 회수)
analyzer.clear_cache();

// 사용 예: 주기적 초기화
let mut last_clear = Instant::now();
loop {
    // ... 처리 ...

    if last_clear.elapsed() > Duration::from_secs(3600) {
        analyzer.clear_cache();
        last_clear = Instant::now();
    }
}
```

## 설정 튜닝 가이드

### 복합명사 분해 모드 선택

| 모드 | 용도 | 예시 |
|------|------|------|
| `None` | 정확한 매칭 필요 | "자연어처리" 검색 시 정확히 일치만 |
| `Discard` | 형태소 단위 검색 | "자연어처리" → "자연어", "처리"만 검색 |
| `Mixed` | 포괄적 검색 | "자연어처리" → 원본 + 분해 모두 검색 |

**권장**: 대부분의 경우 `Mixed` 사용

### Stoptags 선택

```rust
// 기본 (권장)
.with_stoptags(vec!["J".to_string(), "E".to_string()])

// 확장 (더 많이 필터링)
.with_stoptags(vec![
    "J".to_string(),   // 조사
    "E".to_string(),   // 어미
    "SF".to_string(),  // 마침표, 쉼표 등
    "SP".to_string(),  // 쉼표, 가운뎃점 등
    "SS".to_string(),  // 따옴표, 괄호 등
])

// 필터링 없음
.with_stoptags(vec![])
```

### 캐시 크기 결정

| 용도 | 캐시 크기 | 메모리 오버헤드 |
|------|-----------|----------------|
| 검색 엔진 | 1024-2048 | ~200-400 KB |
| 높은 QPS API | 4096-8192 | ~800 KB-1.6 MB |
| 배치 처리 | 0 (비활성화) | 0 |
| IoT/임베디드 | 64-256 | ~12-50 KB |

**계산식**: 메모리 ≈ 캐시_크기 × 200 bytes

## 일반적인 실수

### ❌ 잘못된 사용

```rust
// 매번 새 analyzer 생성 (느림!)
for text in texts {
    let analyzer = NoriAnalyzer::new(config.clone())?;  // ❌
    let tokens = analyzer.analyze(text)?;
}
```

### ✅ 올바른 사용

```rust
// analyzer 재사용 (빠름!)
let analyzer = NoriAnalyzer::new(config)?;  // ✅
for text in texts {
    let tokens = analyzer.analyze(text)?;
}
```

### ❌ 배치 처리 미사용

```rust
// 순차 처리 (느림!)
for doc in &documents {
    let tokens = analyzer.analyze(doc)?;  // ❌
    // process tokens...
}
```

### ✅ 배치 처리 사용

```rust
#[cfg(feature = "batch")]
{
    // 병렬 처리 (빠름!)
    let docs: Vec<&str> = documents.iter().map(|s| s.as_ref()).collect();
    let results = analyzer.analyze_batch(&docs)?;  // ✅
}
```

## Feature Flags

```toml
[dependencies]
mecab-ko-elasticsearch = { version = "0.1", features = ["batch", "cache"] }
```

- `batch`: 병렬 배치 처리 활성화
- `cache`: LRU 캐싱 활성화 (기본값)
- `jni-bindings`: JNI 바인딩 (Java 통합용)
- `async`: Async/await 지원

## 성능 측정

### 벤치마크 실행

```bash
# 기본 벤치마크
cargo bench --bench analyzer_bench

# 성능 벤치마크
cargo bench --bench performance_bench

# 프로파일링
./scripts/profile.sh
```

### 자체 벤치마크 작성

```rust
use std::time::Instant;

let analyzer = NoriAnalyzer::new(config)?;
let iterations = 10000;

let start = Instant::now();
for _ in 0..iterations {
    let _ = analyzer.analyze("테스트 문장")?;
}
let elapsed = start.elapsed();

println!("QPS: {:.0}", iterations as f64 / elapsed.as_secs_f64());
```

## 트러블슈팅

### 느린 성능

**체크리스트**:
1. [ ] Release 모드로 빌드했나요? (`--release`)
2. [ ] Analyzer를 재사용하나요?
3. [ ] 캐시가 활성화되어 있나요?
4. [ ] 배치 처리를 사용하나요? (대량 문서)

### 높은 메모리 사용

**해결책**:
1. 캐시 크기 줄이기: `with_cache_size(config, 256)`
2. 캐시 비활성화: `without_cache(config)`
3. 주기적 캐시 초기화: `analyzer.clear_cache()`

### 낮은 캐시 히트율

**원인**:
- 쿼리가 매번 다름 (정상)
- 캐시가 너무 작음

**해결책**:
- 캐시 크기 증가: `with_cache_size(config, 4096)`
- 또는 캐시 비활성화 (이득 없으면)

## 다음 단계

- 상세 성능 가이드: [PERFORMANCE.md](PERFORMANCE.md)
- API 문서: `cargo doc --open`
- 예제 코드: `examples/` 디렉토리
- 프로파일링: `./scripts/profile.sh`

## 도움말

질문이나 이슈는 GitHub Issues에 등록해주세요.
