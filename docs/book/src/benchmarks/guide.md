# 벤치마크 가이드

MeCab-Ko의 벤치마크를 실행하고 성능을 측정하는 방법을 안내합니다.

## 목차

1. [벤치마크 환경](#벤치마크-환경)
2. [벤치마크 실행](#벤치마크-실행)
3. [벤치마크 종류](#벤치마크-종류)
4. [결과 분석](#결과-분석)
5. [CI 통합](#ci-통합)
6. [커스텀 벤치마크](#커스텀-벤치마크)

## 벤치마크 환경

### 시스템 요구사항

| 항목 | 최소 | 권장 |
|------|------|------|
| CPU | 2 cores | 4+ cores |
| RAM | 4 GB | 8+ GB |
| Rust | 1.70+ | 1.75+ |
| 사전 | mini-dict | full-dict |

### 환경 변수 설정

```bash
# 사전 경로 (선택)
export MECAB_KO_DIC_DIR=/path/to/mecab-ko-dic

# 전체 사전 벤치마크 활성화
export MECAB_KO_FULL_DICT=1

# 벤치마크 반복 횟수
export BENCHMARK_SAMPLES=100
```

## 벤치마크 실행

### 모든 벤치마크 실행

```bash
cd rust/crates/benchmarks
cargo bench
```

### 특정 벤치마크 실행

```bash
# 토크나이저 벤치마크만
cargo bench --bench tokenizer_bench

# 배치 처리 벤치마크만
cargo bench --bench batch_bench

# 특정 테스트만
cargo bench -- "tokenize_short"
```

### 빠른 테스트 모드

```bash
# 샘플 수 줄이기
cargo bench -- --sample-size 10

# 워밍업 시간 줄이기
cargo bench -- --warm-up-time 1
```

### 결과 저장

```bash
# JSON 결과 저장
cargo bench -- --save-baseline v0.2.0

# 이전 결과와 비교
cargo bench -- --baseline v0.1.1
```

## 벤치마크 종류

### 1. tokenizer_bench

기본 토큰화 성능 측정

```rust
// 짧은 텍스트 (5자)
tokenize_short: "안녕하세요"

// 중간 텍스트 (50자)
tokenize_medium: "오늘 날씨가 좋아서 공원에서 산책을 했습니다..."

// 긴 텍스트 (200자)
tokenize_long: "자연어 처리는 인공지능 분야에서..."
```

| 지표 | 설명 |
|------|------|
| time | 평균 실행 시간 |
| throughput | 초당 처리량 (ops/sec) |
| std_dev | 표준 편차 |

### 2. batch_bench

배치 처리 성능

```rust
// 배치 크기별 테스트
batch_100: 100개 문장
batch_1000: 1000개 문장
batch_5000: 5000개 문장
```

| 지표 | 설명 |
|------|------|
| total_time | 전체 처리 시간 |
| avg_per_text | 문장당 평균 시간 |
| throughput | MB/s 처리량 |

### 3. memory_bench

메모리 사용량 측정

```rust
// 측정 항목
memory_allocation: 토큰화 시 메모리 할당
memory_reuse: 재사용 효율성
memory_pressure: 메모리 압력 테스트
```

### 4. cold_start_bench

초기화 시간 측정

```rust
// 측정 항목
dict_load: 사전 로딩 시간
first_tokenize: 첫 번째 토큰화
warm_tokenize: 워밍업 후 토큰화
```

### 5. viterbi_bench

Viterbi 알고리즘 성능

```rust
// 측정 항목
viterbi_search: 최적 경로 탐색
lattice_build: 래티스 구축
path_backtrack: 경로 역추적
```

### 6. trie_bench

Double-Array Trie 검색 성능

```rust
// 측정 항목
exact_match: 정확히 일치 검색
prefix_search: 접두사 검색
batch_lookup: 배치 검색
```

### 7. matrix_bench

연접 비용 매트릭스 검색

```rust
// 측정 항목
single_lookup: 단일 조회
batch_lookup: 배치 조회
cache_hit: 캐시 히트율
```

## 결과 분석

### HTML 리포트

```bash
# 벤치마크 실행 후 리포트 열기
cargo bench
open target/criterion/report/index.html
```

### 리포트 구성

```
target/criterion/
├── report/
│   └── index.html          # 전체 개요
├── tokenize_short/
│   ├── report/index.html   # 개별 벤치마크 상세
│   └── base/               # 기준선 데이터
└── ...
```

### 통계 해석

| 지표 | 의미 |
|------|------|
| Mean | 평균 실행 시간 |
| Std. Dev. | 표준 편차 (낮을수록 안정적) |
| Median | 중앙값 (이상치 영향 적음) |
| MAD | 중앙값 절대 편차 |

### 회귀 감지

```
Performance has regressed:
  tokenize_short time: [5.2 us 5.3 us 5.4 us]
                       change: [+8.2% +10.1% +12.3%] (p = 0.00 < 0.05)
```

- **p < 0.05**: 통계적으로 유의미한 변화
- **change > 10%**: 주의 필요
- **change > 20%**: 회귀 가능성 높음

## CI 통합

### GitHub Actions 워크플로우

```yaml
name: Benchmark

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Run benchmarks
        run: |
          cd rust/crates/benchmarks
          cargo bench -- --save-baseline current

      - name: Compare with main (PR only)
        if: github.event_name == 'pull_request'
        run: |
          git fetch origin main
          git checkout origin/main -- target/criterion
          cargo bench -- --baseline main

      - name: Upload results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/
```

### 회귀 임계값 설정

```yaml
# .github/workflows/benchmark.yml
env:
  BENCHMARK_THRESHOLD_PASS: 5      # 5% 미만: PASS
  BENCHMARK_THRESHOLD_WARN: 10     # 5-10%: WARNING
  BENCHMARK_THRESHOLD_FAIL: 20     # 10% 초과: FAIL
```

### PR 코멘트 생성

```yaml
- name: Post benchmark comment
  uses: actions/github-script@v7
  with:
    script: |
      const fs = require('fs');
      const results = JSON.parse(fs.readFileSync('benchmark-comparison.json'));

      let comment = '## Benchmark Results\n\n';
      comment += '| Benchmark | Before | After | Change |\n';
      comment += '|-----------|--------|-------|--------|\n';

      for (const [name, data] of Object.entries(results)) {
        const change = ((data.after - data.before) / data.before * 100).toFixed(1);
        const emoji = change > 10 ? '🔴' : change > 5 ? '🟡' : '🟢';
        comment += `| ${name} | ${data.before}us | ${data.after}us | ${emoji} ${change}% |\n`;
      }

      github.rest.issues.createComment({
        owner: context.repo.owner,
        repo: context.repo.repo,
        issue_number: context.issue.number,
        body: comment
      });
```

## 커스텀 벤치마크

### Criterion 벤치마크 작성

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mecab_ko::Tokenizer;

fn custom_benchmark(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // 단일 벤치마크
    c.bench_function("my_custom_bench", |b| {
        b.iter(|| {
            tokenizer.tokenize(black_box("테스트 문장"))
        })
    });

    // 파라미터화된 벤치마크
    let texts = vec![
        ("short", "안녕"),
        ("medium", "오늘 날씨가 좋습니다"),
        ("long", "자연어 처리는 인공지능의 핵심 기술입니다"),
    ];

    let mut group = c.benchmark_group("text_length");
    for (name, text) in texts {
        group.bench_with_input(
            BenchmarkId::new("tokenize", name),
            &text,
            |b, text| {
                b.iter(|| tokenizer.tokenize(black_box(text)))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, custom_benchmark);
criterion_main!(benches);
```

### 처리량 측정

```rust
use criterion::{Criterion, Throughput};

fn throughput_benchmark(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().unwrap();
    let text = "벤치마크 테스트용 긴 텍스트...".repeat(100);

    let mut group = c.benchmark_group("throughput");
    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("large_text", |b| {
        b.iter(|| tokenizer.tokenize(black_box(&text)))
    });

    group.finish();
}
```

### 메모리 측정

```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

struct CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

fn memory_benchmark(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().unwrap();

    c.bench_function("memory_usage", |b| {
        b.iter_custom(|iters| {
            let start = ALLOCATED.load(Ordering::SeqCst);

            for _ in 0..iters {
                let _ = tokenizer.tokenize("테스트");
            }

            let end = ALLOCATED.load(Ordering::SeqCst);
            std::time::Duration::from_nanos(
                ((end - start) / iters as usize) as u64
            )
        })
    });
}
```

## 성능 최적화 팁

### 벤치마크 시 주의사항

1. **릴리스 빌드 사용**
   ```bash
   cargo bench  # 기본적으로 --release
   ```

2. **시스템 부하 최소화**
   - 다른 프로그램 종료
   - 백그라운드 프로세스 최소화

3. **충분한 워밍업**
   ```rust
   criterion_group! {
       name = benches;
       config = Criterion::default()
           .warm_up_time(std::time::Duration::from_secs(5))
           .measurement_time(std::time::Duration::from_secs(10));
       targets = my_benchmark
   }
   ```

4. **통계적 유의성 확인**
   - 최소 100회 이상 측정
   - p-value < 0.05 확인

### 일반적인 성능 문제

| 문제 | 원인 | 해결책 |
|------|------|--------|
| Cold start 느림 | 사전 로딩 | mmap 사용 |
| 메모리 과다 사용 | 캐싱 없음 | LRU 캐시 적용 |
| 배치 처리 느림 | 직렬 처리 | Rayon 병렬화 |
| 긴 텍스트 느림 | O(n^2) 알고리즘 | 청크 분할 |

## 벤치마크 결과 예시

### v0.2.0 기준선

```
tokenize_short    time: [3.8 us 3.9 us 4.0 us]
tokenize_medium   time: [42.1 us 43.2 us 44.5 us]
tokenize_long     time: [138.2 us 141.5 us 145.1 us]

batch_100         time: [2.42 ms 2.48 ms 2.55 ms]
batch_1000        time: [26.8 ms 27.4 ms 28.1 ms]

cold_start        time: [128.5 ms 132.1 ms 136.2 ms]
```

### KPI 목표

| 지표 | 목표 | 현재 | 상태 |
|------|------|------|------|
| Throughput | 200K ops/sec | 238K | PASS |
| Cold Start | < 200ms | 132ms | PASS |
| Memory | < 150MB | 145MB | PASS |

## 참고 자료

- [Criterion.rs 문서](https://bheisler.github.io/criterion.rs/book/)
- [Rust 성능 책](https://nnethercote.github.io/perf-book/)
- [성능 튜닝 가이드](../advanced/performance-tuning.md)
- [성능 대시보드](index.md)
