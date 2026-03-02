# Benchmark Results

Latest performance measurements for MeCab-Ko v0.2.0.

**Last Updated:** 2026-03-02
**Platform:** Linux 6.8.0-83-generic, x86_64 (CI) / macOS Darwin 25.3.0 (full dict)
**Dictionary:** mini-dict (CI) / mecab-ko-dic 2.1.1 (full dict measurements)

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [KPI Dashboard](#kpi-dashboard)
3. [Tokenization Performance](#tokenization-performance)
4. [Cold Start Performance](#cold-start-performance)
5. [Batch Processing](#batch-processing)
6. [Memory Performance](#memory-performance)
7. [Algorithm Benchmarks](#algorithm-benchmarks)
8. [Scalability Analysis](#scalability-analysis)
9. [Competitor Comparison](#competitor-comparison)
10. [Historical Trends](#historical-trends)

---

## Executive Summary

MeCab-Ko demonstrates strong tokenization performance with sub-millisecond latency for typical inputs. Key findings:

- **Throughput:** ~238K morphemes/sec (exceeds 150K target)
- **Cold start:** 0.13s (within 0.2s target)
- **Memory:** 215 MB peak (exceeds 150 MB target - optimization needed)
- **Scalability:** Linear to 100 chars, super-linear beyond (optimization opportunity)

### Performance Highlights

```
+------------------+--------+---------+--------+
|     Metric       | Target | Actual  | Status |
+------------------+--------+---------+--------+
| Morphemes/sec    | 150K   | ~238K   |   OK   |
| Cold start       | <200ms | 130ms   |   OK   |
| Peak memory      | <150MB | 215MB   |  WARN  |
| Short text (<10) | <20us  | 5.8us   |   OK   |
| Medium text      | <100us | 42us    |   OK   |
+------------------+--------+---------+--------+
```

---

## KPI Dashboard

### Primary KPIs

| KPI | Target | Measured | Variance | Status |
|-----|--------|----------|----------|--------|
| Morphemes/sec | 150,000 | ~238,000 | +59% | PASS |
| Cold start time | < 200ms | 130ms | -35% | PASS |
| Memory per instance | < 150MB | 215MB | +43% | **OVER** |

### Secondary KPIs

| KPI | Target | Measured | Status |
|-----|--------|----------|--------|
| Short text latency | < 20us | 5.8us | PASS |
| Medium text latency | < 100us | 42us | PASS |
| Long text latency | < 1ms | 136us | PASS |
| Batch throughput | > 3 MiB/s | 1.96 MiB/s | NEEDS WORK |
| Dictionary load time | < 100ms | ~60us | PASS |

---

## Tokenization Performance

### Basic Performance

| Input Type | Characters | Time (us) | Throughput | Tokens |
|------------|------------|-----------|------------|--------|
| Short | ~5 | 5.81 | - | ~3-5 |
| Medium | ~50 | 42.16 | 1.7 MiB/s | ~15-20 |
| Long | ~200 | 135.51 | 1.5 MiB/s | ~50-70 |

### By Text Type

| Text Type | Time (us) | Notes |
|-----------|-----------|-------|
| General | 37.79 | Standard written Korean |
| Technical | 60.38 | Mixed Korean/English |
| Mixed | 49.72 | Numbers, symbols |

### By Processing Mode

| Mode | Time (us) | Use Case |
|------|-----------|----------|
| tokenize() | 42.16 | Full analysis |
| wakati() | 39.53 | Surface forms only |
| pos() | 39.66 | POS tags only |
| nouns() | 37.30 | Noun extraction |

### Realistic Workloads

| Scenario | Time (us) | Description |
|----------|-----------|-------------|
| Social media | 39.99 | Short posts batch |
| Document indexing | 197.77 | Noun extraction batch |
| News article | 536.25 | Long text analysis |

---

## Cold Start Performance

### Initialization Timing

```
+----------------------------+----------+
|         Operation          | Time(us) |
+----------------------------+----------+
| Full initialization        |    59.96 |
| Init + first tokenize      |    85.90 |
| First tokenization (cold)  |    45.74 |
| Warmed tokenization        |    37.14 |
| Heavily warmed (10x)       |    39.73 |
| Recreate each time         |    80.61 |
| Reuse tokenizer            |    10.41 |
+----------------------------+----------+
```

### Cache Warming Effect

| Warm-up Level | Time (us) | Speedup vs Cold |
|---------------|-----------|-----------------|
| Cold (0 iterations) | 45.74 | 1.00x |
| Warmed (1 iteration) | 37.14 | 1.23x |
| Heavily warmed (10) | 39.73 | 1.15x |

### Key Insight

**Tokenizer reuse is critical:** Reusing tokenizer (10.41us) is **7.7x faster** than recreating each time (80.61us).

---

## Batch Processing

### By Batch Size

| Batch Size | Total Time (ms) | Per-Text (us) | Throughput |
|------------|-----------------|---------------|------------|
| 1 | 0.01 | 10.73 | - |
| 5 | 0.11 | 22.33 | - |
| 10 | 0.24 | 24.22 | - |
| 20 | 0.50 | 25.13 | - |
| 50 | 1.23 | 24.64 | - |
| 100 | 2.47 | 24.75 | - |
| 200 | 4.99 | 24.94 | - |
| 500 | 12.86 | 25.73 | - |
| 1,000 | 26.84 | 26.84 | - |
| 2,000 | 51.69 | 25.84 | - |
| 5,000 | 125.65 | 25.13 | - |

### Specialized Scenarios

| Scenario | Texts | Total Time | Throughput |
|----------|-------|------------|------------|
| News articles (100) | 100 | 5.72ms | 1.64 MiB/s |
| Social media (1000) | 1000 | 8.48ms | - |
| Streaming (collect) | 100 | 2.52ms | 1.96 MiB/s |
| Streaming (process) | 100 | 2.55ms | 1.94 MiB/s |

### Throughput Visualization

```
Batch Throughput (texts/sec)

  5000 |----------------------------------------* 39,763
  2000 |------------------------* 38,693        |
  1000 |------------* 37,257    |               |
   500 |------* 38,880          |               |
   100 |--* 40,407              |               |
    50 |-* 40,584               |               |
       +-------+-------+-------+-------+-------+
              20K     30K     40K     50K (texts/sec)
```

---

## Memory Performance

### Per-Tokenization Memory

| Text Length | Time (us) | Relative |
|-------------|-----------|----------|
| Short | 5.42 | 1.0x |
| Medium | 43.17 | 8.0x |
| Long | 188.44 | 34.8x |

### Memory Scalability

| Chars | Time (us) | us/char |
|-------|-----------|---------|
| 10 | 16.75 | 1.68 |
| 50 | 102.86 | 2.06 |
| 100 | 240.30 | 2.40 |
| 500 | 1,975.37 | 3.95 |
| 1,000 | 5,673.14 | 5.67 |
| 5,000 | 99,254.02 | 19.85 |

### Memory Patterns

| Pattern | Time (us) | Description |
|---------|-----------|-------------|
| Immediate drop | 13,077.80 | Drop each result |
| Batch drop | 12,642.86 | Accumulate then drop |
| Chunked processing | 8,263.67 | Process in chunks |
| All at once | 29,266.40 | Single large text |

**Key Insight:** Processing in **chunks is 3.5x faster** than processing large texts all at once.

### Full Dictionary Memory (macOS)

| Metric | Value | Target | Status |
|--------|-------|--------|--------|
| Peak Memory Footprint | 211 MB | < 150 MB | OVER |
| Maximum RSS | 215 MB | < 150 MB | OVER |
| On-disk dictionary | 93 MB | - | - |
| Memory amplification | 2.3x | < 1.6x | OVER |

**Memory Breakdown (estimated):**
- Trie structure: ~40 MB
- Connection matrix: ~50 MB
- Entry data: ~100 MB
- Runtime overhead: ~25 MB

---

## Algorithm Benchmarks

### Viterbi Search

| Lattice Size | Time (us) | Description |
|--------------|-----------|-------------|
| Small (5 nodes) | 0.71 | Linear path |
| Medium (7 nodes) | 5.65 | Multiple paths |
| Large (50+ nodes) | 28.22 | Complex lattice |

### N-best Search

| N | Time (us) | Overhead vs N=1 |
|---|-----------|-----------------|
| 1 | 4.5 | 1.0x |
| 3 | 6.2 | 1.4x |
| 5 | 8.1 | 1.8x |
| 10 | 12.4 | 2.8x |

### Space Penalty Overhead

| Configuration | Time (us) | Overhead |
|---------------|-----------|----------|
| No penalty | 5.1 | 1.0x |
| Korean default | 5.3 | 1.04x |
| Custom (50 entries) | 5.5 | 1.08x |

### Trie Operations

| Operation | Time (ns) | Description |
|-----------|-----------|-------------|
| exact_match (hit) | 60 | Key found |
| exact_match (miss) | 10 | Key not found |
| common_prefix_search | 100-110 | Prefix matching |
| build (1000 entries) | 31,170 | Construction |

### Matrix Operations

| Operation | Time (ns) | Description |
|-----------|-----------|-------------|
| Single lookup (fixed) | 2 | Direct access |
| Single lookup (random) | 20 | Random access |
| Batch lookup (1000) | 2,480 | Bulk retrieval |

---

## Scalability Analysis

### Input Size Scaling

```
Time vs Input Size

Time |                                         *
(ms) |                                    *****
  10 |                               *****
     |                          *****
   1 |                    ******
     |              ******
 0.1 |        ******
     |  ******
0.01 |**
     +--+----+----+----+----+----+----+----+----+
        10   50  100  200  500  1K   2K   5K  10K
                     Characters
```

| Characters | Time (us) | us/char | Scaling |
|------------|-----------|---------|---------|
| 10 | 8.63 | 0.86 | Linear |
| 50 | 77.53 | 1.55 | Linear |
| 100 | 198.17 | 1.98 | Linear |
| 500 | 3,054.60 | 6.11 | Super-linear |
| 1000 | 9,977.94 | 9.98 | Super-linear |

**Observation:** Scaling is linear up to ~100 characters, then becomes super-linear. This indicates optimization opportunities in lattice building or Viterbi search for longer texts.

### Batch Size Scaling

Batch processing shows consistent per-text overhead (~25us) across batch sizes, indicating good scalability.

---

## Competitor Comparison

### Overview Comparison

| Analyzer | Language | Throughput | Memory | Dictionary |
|----------|----------|------------|--------|------------|
| MeCab-Ko (C++) | C++ | 18 MB/s | ~80 MB | mmap-based |
| **MeCab-Ko (Rust)** | **Rust** | **15 MB/s** | **215 MB** | **in-memory** |
| Kiwi | C++ | 22 MB/s | ~150 MB | Custom model |
| Nori (Lucene) | Java | ~10 MB/s | Variable | Embedded |
| Lindera | Rust | 12 MB/s | ~180 MB | MeCab format |

### Feature Comparison

| Feature | MeCab-Ko (Rust) | Kiwi | Nori |
|---------|-----------------|------|------|
| Korean optimization | Yes | Yes | Yes |
| Elasticsearch integration | Yes | No | Native |
| User dictionary | Yes | Yes | Yes |
| N-best output | Yes | Yes | No |
| WASM support | Yes | No | No |
| Memory mapping | Partial | Yes | N/A |

### Latency Comparison (estimated)

```
Short Text Latency (lower is better)
+------------------+
| Kiwi        | ** |  ~4us
| MeCab (C++) | ** |  ~5us
| MeCab (Rust)| ***|  ~6us
| Lindera     |****|  ~8us
| Nori        |****|  ~10us
+------------------+
```

---

## Historical Trends

### Performance Improvements (Sprint 6)

| Input Size | Before | After | Improvement |
|------------|--------|-------|-------------|
| 10 chars | 8.6us | 3.8us | -55% |
| 50 chars | 77.5us | 44.9us | -42% |
| 100 chars | 198us | 141us | -31% |
| 500 chars | 3055us | 2165us | -29% |
| 1000 chars | 9978us | 8413us | -16% |

### Optimization Techniques Applied

1. **SpacePositions:** HashSet -> sorted Vec + binary_search
2. **SpacePenalty:** linear scan -> binary_search
3. **Feature parsing:** Vec allocation -> splitn iterator
4. **Lattice:** Added byte_to_char binary search helper

### Memory Trend

| Version | Peak Memory | Dictionary |
|---------|-------------|------------|
| v0.1.0 | ~250 MB | In-memory |
| v0.1.1 | ~215 MB | Partial mmap |
| v0.2.0 | ~215 MB | Lazy loading |

---

## Optimization Targets

Based on benchmark analysis, priority optimization areas:

### High Priority

1. **Memory reduction** (215 MB -> 150 MB target)
   - Implement full lazy loading for entries
   - Expand mmap usage for matrix
   - Consider string interning

2. **Long text scalability**
   - Investigate O(n^2) behavior above 100 chars
   - Optimize lattice building for long texts
   - Consider chunked processing in tokenizer

### Medium Priority

3. **Batch throughput** (1.96 MiB/s -> 3 MiB/s target)
   - Parallel batch processing
   - Pre-allocated result buffers

4. **Cold start reduction**
   - Lazy dictionary loading
   - Faster deserialization

### Low Priority

5. **Algorithm micro-optimizations**
   - SIMD for matrix lookups
   - Cache-optimized data layouts

---

## Appendix: Raw Data

### Test Environment Details

```
CI Environment:
- GitHub Actions ubuntu-latest
- 2 vCPUs, 7GB RAM
- Rust stable

Full Dict Environment:
- macOS Darwin 25.3.0
- Apple Silicon (arm64)
- Rust stable
- mecab-ko-dic 2.1.1 (816,283 entries)
```

### Benchmark Commands

```bash
# Run all benchmarks
cd rust/crates/benchmarks && cargo bench

# Run with full dictionary
MECAB_KO_FULL_DICT=1 cargo bench

# Save baseline
cargo bench -- --save-baseline v0.2.0

# Compare to baseline
cargo bench -- --baseline v0.2.0
```

---

## See Also

- [Methodology](./methodology.md) - How measurements are conducted
- [Performance Tuning Guide](../book/src/advanced/performance-tuning.md) - Optimization tips
- [Sprint 6 Baseline](../research/benchmarks/sprint6-baseline.md) - Historical data
- [Sprint 7 Memory KPI](../research/benchmarks/sprint7-memory-kpi.md) - Memory analysis
