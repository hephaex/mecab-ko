# MeCab-Ko Performance Benchmarks

Performance benchmarking documentation for the MeCab-Ko Korean morphological analyzer.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Key Performance Metrics](#key-performance-metrics)
4. [Documentation Structure](#documentation-structure)
5. [Running Benchmarks](#running-benchmarks)

## Overview

MeCab-Ko provides comprehensive benchmarking infrastructure built on [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) for accurate, statistically-sound performance measurements. The benchmark suite covers:

- **Tokenization throughput** - Characters and tokens processed per second
- **Latency** - Response time by input size
- **Memory usage** - Peak and sustained memory consumption
- **Cold start time** - Initialization and first-request latency
- **Scalability** - Performance characteristics across input sizes

## Quick Start

```bash
# Run all benchmarks
cd rust/crates/benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench tokenizer_bench

# Quick test mode (fewer samples)
cargo bench -- --sample-size 10

# View HTML report
open target/criterion/report/index.html
```

## Key Performance Metrics

### Current KPIs (v0.2.0)

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| Morphemes/sec | 150,000 | ~238,000 | PASS |
| Cold start | < 200ms | 0.13s | PASS |
| Peak memory (full dict) | < 150MB | 215MB | NEEDS OPTIMIZATION |

### Performance Summary

```
Tokenization Performance (mini-dict, CI environment)
-----------------------------------------------------
Short text (5 chars):    5.8 us     (~1.7 MB/s)
Medium text (50 chars):  42 us      (~1.7 MB/s)
Long text (200 chars):   136 us     (~1.5 MB/s)

Batch Processing
-----------------------------------------------------
100 texts:               2.5 ms     (~1.96 MiB/s)
1000 texts:              27 ms      (~1.85 MiB/s)
5000 texts:              126 ms     (~1.78 MiB/s)
```

## Documentation Structure

| Document | Description |
|----------|-------------|
| [README.md](./README.md) | This overview document |
| [methodology.md](./methodology.md) | Measurement methodology and test environment |
| [results.md](./results.md) | Latest benchmark results with detailed analysis |

## Running Benchmarks

### Prerequisites

- Rust 1.70+ (stable)
- MeCab-Ko dictionary (mini-dict or full dict)
- At least 4GB RAM for full dictionary benchmarks

### Environment Setup

```bash
# Set dictionary path (optional, auto-detected)
export MECAB_KO_DIC_DIR=/path/to/mecab-ko-dic

# For full dictionary benchmarks
export MECAB_KO_FULL_DICT=1
```

### Available Benchmark Suites

| Suite | Description | Command |
|-------|-------------|---------|
| `tokenizer_bench` | End-to-end tokenization | `cargo bench --bench tokenizer_bench` |
| `batch_bench` | Batch processing throughput | `cargo bench --bench batch_bench` |
| `memory_bench` | Memory allocation patterns | `cargo bench --bench memory_bench` |
| `cold_start_bench` | Initialization time | `cargo bench --bench cold_start_bench` |
| `viterbi_bench` | Viterbi algorithm | `cargo bench --bench viterbi_bench` |
| `trie_bench` | Dictionary lookup | `cargo bench --bench trie_bench` |
| `matrix_bench` | Connection cost matrix | `cargo bench --bench matrix_bench` |
| `normalization_bench` | Text normalization | `cargo bench --bench normalization_bench` |
| `comparison_bench` | Mode comparison | `cargo bench --bench comparison_bench` |

### Saving Baselines

```bash
# Save current results as baseline
cargo bench -- --save-baseline v0.2.0

# Compare against baseline after changes
cargo bench -- --baseline v0.2.0
```

## Competitor Comparison

For competitive analysis against other Korean analyzers, see [results.md](./results.md#competitor-comparison).

| Analyzer | Throughput | Memory | Notes |
|----------|------------|--------|-------|
| MeCab-Ko (C++) | 18 MB/s | ~80 MB | mmap-based |
| **MeCab-Ko (Rust)** | **15 MB/s** | **215 MB** | Current implementation |
| Kiwi | 22 MB/s | ~150 MB | C++, optimized |
| Nori (Lucene) | ~10 MB/s | Variable | Java, Elasticsearch |

## CI Integration

Benchmarks run automatically on:
- Push to `main` branch
- Pull requests (comparison mode)
- Manual workflow dispatch

Results are stored as GitHub Actions artifacts and summarized in PR comments.

## See Also

- [Methodology](./methodology.md) - How benchmarks are conducted
- [Results](./results.md) - Detailed performance data
- [Performance Tuning Guide](../book/src/advanced/performance-tuning.md) - Optimization tips
