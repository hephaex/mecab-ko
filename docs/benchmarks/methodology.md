# Benchmark Methodology

This document describes the methodology used for MeCab-Ko performance benchmarks.

## Table of Contents

1. [Test Environment](#test-environment)
2. [Measurement Framework](#measurement-framework)
3. [Benchmark Categories](#benchmark-categories)
4. [Input Data](#input-data)
5. [Statistical Analysis](#statistical-analysis)
6. [Reproducibility](#reproducibility)

## Test Environment

### Reference Hardware

Primary benchmarks are conducted on:

| Component | Specification |
|-----------|---------------|
| CPU | AMD Ryzen 9 5950X (16 cores, 32 threads) |
| RAM | 64GB DDR4-3200 |
| Storage | NVMe SSD |
| OS | Ubuntu 22.04 LTS / macOS 14+ |
| Rust | stable (latest) |

### CI Environment

Automated benchmarks run on GitHub Actions:

| Component | Specification |
|-----------|---------------|
| CPU | 2 vCPUs (x86_64) |
| RAM | 7GB |
| OS | Ubuntu 22.04 |
| Dictionary | mini-dict (CI fixture) |

### Dictionary Configurations

| Configuration | Entries | Size | Use Case |
|---------------|---------|------|----------|
| **mini-dict** | ~1,000 | ~1 MB | CI, quick tests |
| **full-dict** | 816,283 | 93 MB | Production benchmarks |

## Measurement Framework

### Criterion.rs

All benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) for:

- Statistical sampling with configurable sample sizes
- Automatic outlier detection and removal
- Regression detection between runs
- HTML report generation with visualizations

### Key Configuration

```rust
// Default benchmark configuration
criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(100)        // 100 iterations per benchmark
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(3));
    targets = bench_function
}
```

### Measurement Types

| Type | Unit | Description |
|------|------|-------------|
| **Time** | ns, us, ms | Execution time per operation |
| **Throughput** | bytes/sec, elements/sec | Data processing rate |
| **Memory** | bytes | Allocation size |

## Benchmark Categories

### 1. Tokenization Performance

Measures end-to-end tokenization throughput and latency.

```
Benchmarks:
- tokenizer_basic: Short/medium/long single texts
- tokenizer_batch: Batch processing of multiple texts
- tokenizer_scalability: Performance vs input size
- tokenizer_text_types: Different content types
- tokenizer_realistic_workload: Real-world scenarios
```

**Methodology:**
- Input: Pre-defined test sentences of varying lengths
- Measure: Time to complete tokenize() call
- Report: Time per call, throughput in bytes/sec

### 2. Cold Start / Initialization

Measures startup overhead and cache warming effects.

```
Benchmarks:
- full_initialization: Tokenizer creation with dictionary loading
- init_plus_first_tokenize: Initialization + first request
- first_tokenization: Cold cache performance
- warmed_tokenization: Warm cache performance
- reuse_vs_recreate: Tokenizer reuse efficiency
```

**Methodology:**
- Use `iter_batched()` with `BatchSize::LargeInput` for cold measurements
- Compare cold vs warm performance ratios
- Measure cumulative warm-up effect (1, 10, 100 iterations)

### 3. Batch Processing

Measures throughput for multiple texts.

```
Benchmarks:
- batch_small: 1-50 texts
- batch_medium: 100-500 texts
- batch_large: 1000-5000 texts
- batch_mixed_length: Variable length inputs
- batch_streaming_vs_collect: Processing strategies
```

**Methodology:**
- Generate batches of N texts from templates
- Measure total time for processing all texts
- Report: texts/sec, bytes/sec, time per text

### 4. Memory Usage

Analyzes memory allocation patterns.

```
Benchmarks:
- per_tokenization_memory: Allocation per call
- memory_reuse: Sequential tokenization efficiency
- memory_accumulation: Memory growth patterns
- memory_scalability: Memory vs input size
- memory_pressure: Performance under allocation stress
```

**Methodology:**
- Use timing as proxy for allocation overhead
- Compare immediate drop vs batch drop patterns
- Analyze chunked vs monolithic processing

### 5. Algorithm-Specific

#### Viterbi Algorithm
```
Benchmarks:
- viterbi_search: Path finding performance
- space_penalty: Penalty computation overhead
- nbest_search: N-best path extraction
- scalability_by_nodes: Performance vs lattice size
```

#### Trie Operations
```
Benchmarks:
- exact_match: Single key lookup
- common_prefix_search: Prefix matching
- build: Trie construction time
```

#### Matrix Operations
```
Benchmarks:
- single_lookup: Individual cost retrieval
- batch_lookup: Bulk cost retrieval
- cache_locality: Memory access patterns
- viterbi_pattern: Realistic access pattern
```

## Input Data

### Test Sentences

Sentences are categorized by length and type:

```rust
// Short (5-10 chars) - Social media style
const SHORT_SENTENCES: &[&str] = &[
    "안녕하세요",
    "오늘 날씨 좋네요",
    "감사합니다",
];

// Medium (20-50 chars) - General conversation
const MEDIUM_SENTENCES: &[&str] = &[
    "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다",
    "아버지가 방에 들어가신다는 문장을 분석해보겠습니다",
];

// Long (100+ chars) - News article style
const LONG_SENTENCES: &[&str] = &[
    "대한민국의 수도인 서울은 조선시대부터 600년이 넘는 역사를 가진 도시...",
];
```

### Text Types

| Type | Characteristics | Example |
|------|-----------------|---------|
| General | Standard written Korean | News, books |
| Technical | Mixed Korean/English, terminology | Documentation |
| Social Media | Informal, abbreviations | SNS posts |
| Mixed | Korean + numbers + symbols | Addresses, IDs |

### Scalability Testing

Input sizes for scalability analysis:

```
Character counts: 10, 50, 100, 500, 1000, 5000
Batch sizes: 1, 5, 10, 20, 50, 100, 200, 500, 1000, 2000, 5000
```

## Statistical Analysis

### Metrics Reported

| Metric | Description |
|--------|-------------|
| Mean | Average execution time |
| Median | 50th percentile |
| Std Dev | Standard deviation |
| MAD | Median Absolute Deviation |
| Slope | Regression line slope |
| R^2 | Coefficient of determination |

### Outlier Detection

Criterion uses the modified Z-score method:
- Values with |Z| > 3.5 are marked as outliers
- Outliers are reported but excluded from statistics

### Regression Detection

Changes are classified as:
- **Improvement**: >5% faster, statistically significant
- **Regression**: >5% slower, statistically significant
- **No change**: Within noise threshold

## Reproducibility

### Factors Affecting Results

| Factor | Impact | Mitigation |
|--------|--------|------------|
| CPU frequency scaling | High | Disable turbo, lock frequency |
| Background processes | Medium | Run on idle system |
| Memory pressure | Medium | Close other applications |
| Disk I/O | Low-Medium | Use SSD, warm file cache |
| JIT/caching | Medium | Use warm-up iterations |

### Recommended Procedure

```bash
# 1. Ensure consistent CPU frequency (Linux)
sudo cpupower frequency-set -g performance

# 2. Close unnecessary applications
# 3. Run benchmarks
cargo bench -- --save-baseline $(date +%Y%m%d)

# 4. Run multiple times and compare
cargo bench -- --baseline $(date +%Y%m%d)
```

### CI Reproducibility

For CI environments:
- Use `--sample-size 50` for faster runs
- Accept ~10% variance due to shared infrastructure
- Focus on relative comparisons vs absolute values
- Store artifacts for cross-run comparison

## Performance Targets

### Current Goals

| Metric | Target | Rationale |
|--------|--------|-----------|
| Short text latency | < 20 us | Real-time query processing |
| Medium text latency | < 100 us | Interactive response |
| Long text latency | < 1 ms | Document processing |
| Throughput | > 3 MiB/s | Batch processing efficiency |
| Cold start | < 200 ms | Server startup time |
| Memory/instance | < 150 MB | Multi-tenant deployment |

### Future Goals

- Parallel processing: >180 MB/s on 16 cores
- Memory optimization: <100 MB per instance
- WASM bundle size: <5 MB

## See Also

- [Results](./results.md) - Latest benchmark data
- [Performance Tuning](../book/src/advanced/performance-tuning.md) - Optimization guide
- [Criterion Documentation](https://bheisler.github.io/criterion.rs/book/)
