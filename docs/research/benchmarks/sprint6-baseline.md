# Sprint 6 Benchmark Baseline

Date: 2026-02-24
Platform: Linux 6.8.0-83-generic, x86_64
Rust: stable
Dictionary: mini-dict (CI fixture)
Note: Results with mini-dict. Full dictionary benchmarks require MECAB_KO_FULL_DICT.

## KPI Summary

| KPI | Target | Measured | Status |
|-----|--------|----------|--------|
| Morphemes/sec | 150K | ~238K | PASS |
| Cold start | < 200ms | 0.086ms | PASS |
| Memory per instance | < 150MB | N/A (mini-dict) | TBD |

## Tokenizer Benchmarks

### Basic Performance
| Benchmark | Time (µs) | Throughput |
|-----------|-----------|------------|
| short_single | 5.81 | - |
| medium_single | 42.16 | ~1.7 MiB/s |
| long_single | 135.51 | - |

### Realistic Workloads
| Benchmark | Time (µs) | Throughput |
|-----------|-----------|------------|
| social_media | 39.99 | - |
| document_indexing | 197.77 | - |
| news_article | 536.25 | - |

### Scalability (by input char count)
| Chars | Time (µs) | µs/char |
|-------|-----------|---------|
| 10 | 8.63 | 0.863 |
| 50 | 77.53 | 1.551 |
| 100 | 198.17 | 1.982 |
| 500 | 3,054.60 | 6.109 |
| 1000 | 9,977.94 | 9.978 |

Note: Super-linear scaling above 100 chars suggests potential optimization targets in lattice building or Viterbi search.

### Edge Cases
| Benchmark | Time (µs) |
|-----------|-----------|
| empty | 0.01 |
| single_char | 1.66 |
| whitespace_only | 0.33 |
| numbers_only | 34.79 |
| english_only | 35.44 |

### Text Types
| Benchmark | Time (µs) |
|-----------|-----------|
| general | 37.79 |
| technical | 60.38 |
| mixed | 49.72 |

### Throughput
| Benchmark | Time (µs) |
|-----------|-----------|
| high_volume (20 texts) | 772.94 |
| tokenizer_wakati/medium | 39.53 |
| tokenizer_nouns/medium | 37.30 |
| tokenizer_pos/medium | 39.66 |

### Batch Operations
| Benchmark | Time (µs) |
|-----------|-----------|
| short_batch (5 texts) | 40.35 |
| medium_batch (5 texts) | 185.94 |

### Baseline (5 standard sentences)
| Sentence | Time (µs) | Throughput (MiB/s) |
|----------|-----------|-------------------|
| 0 | 41.16 | 1.70 |
| 1 | 38.36 | 1.77 |
| 2 | 35.56 | 1.86 |
| 3 | 35.92 | 1.68 |
| 4 | 44.90 | 1.63 |

## Cold Start Benchmarks

| Benchmark | Time (µs) |
|-----------|-----------|
| full_initialization | 59.96 |
| init_plus_first_tokenize | 85.90 |
| first_tokenization | 45.74 |
| warmed_tokenization | 37.14 |
| heavily_warmed | 39.73 |
| reuse_tokenizer | 10.41 |
| recreate_each_time | 80.61 |
| basic_init | 64.42 |
| init_and_drop | 61.65 |
| sequential_init_3 | 213.31 |
| first_five_requests | 66.43 |
| warmed_five_requests | 55.62 |

## Batch Processing Benchmarks

| Benchmark | Time (µs) | Throughput |
|-----------|-----------|------------|
| batch_small/1 | 10.73 | - |
| batch_small/5 | 111.64 | - |
| batch_small/10 | 242.15 | - |
| batch_small/20 | 502.55 | - |
| batch_small/50 | 1,232.08 | - |
| batch_medium/100 | 2,474.82 | - |
| batch_medium/200 | 4,988.21 | - |
| batch_medium/500 | 12,864.49 | - |
| batch_large/1000 | 26,836.49 | - |
| batch_large/2000 | 51,688.57 | - |
| batch_large/5000 | 125,652.94 | - |
| batch_news_articles/100 | 5,721.90 | 1.64 MiB/s |
| batch_social_media/1000 | 8,482.28 | - |
| batch_streaming/collect_all | 2,516.55 | 1.96 MiB/s |
| batch_streaming/stream_process | 2,547.91 | 1.94 MiB/s |

## Trie Benchmarks

| Benchmark | Time (µs) |
|-----------|-----------|
| build/small_sorted | 31.17 |
| build/small_unsorted | 30.57 |
| build/medium | 419.31 |
| common_prefix_search/multi_match | 0.10 |
| common_prefix_search/long_text | 0.11 |
| common_prefix_search/no_match | 0.01 |
| exact_match/hit | 0.06 |
| exact_match/miss | 0.01 |

## Matrix Benchmarks

| Benchmark | Time (µs) |
|-----------|-----------|
| single_lookup/fixed | 0.002 |
| single_lookup/random | 0.02 |
| single_lookup/sequential | 0.01 |
| batch_lookup/10 | 0.02 |
| batch_lookup/100 | 0.20 |
| batch_lookup/1000 | 2.48 |
| cache_locality/row_major | 1.03 |
| cache_locality/column_major | 1.08 |
| cache_locality/strided | 0.13 |
| size_comparison/small_100x100 | 1.08 |
| size_comparison/medium_1000x1000 | 1.36 |
| size_comparison/large_2000x2000 | 1.52 |
| viterbi_pattern/node_transition | 0.10 |
| viterbi_pattern/path_calculation | 0.89 |
| realistic_workload/sentence_10words | 0.57 |
| realistic_workload/sentence_50words | 2.95 |
| memory/100 | 0.95 |
| memory/500 | 40.07 |
| memory/1000 | 157.76 |
| memory/2000 | 640.41 |

## Memory Benchmarks

| Benchmark | Time (µs) |
|-----------|-----------|
| per_tokenization/short | 5.42 |
| per_tokenization/medium | 43.17 |
| per_tokenization/long | 188.44 |
| reuse/single_allocation | 11.56 |
| reuse/sequential_10 | 114.43 |
| reuse/sequential_100 | 1,142.41 |
| scalability/10 | 16.75 |
| scalability/50 | 102.86 |
| scalability/100 | 240.30 |
| scalability/500 | 1,975.37 |
| scalability/1000 | 5,673.14 |
| scalability/5000 | 99,254.02 |
| tokenizer_instance/single | 68.17 |
| tokenizer_instance/three | 214.82 |
| accumulation/batch_drop | 12,642.86 |
| accumulation/immediate_drop | 13,077.80 |
| long_text_stream/all_at_once | 29,266.40 |
| long_text_stream/in_chunks | 8,263.67 |
| pressure/normal | 36.53 |
| pressure/with_temp_alloc | 35.53 |
| web_server/per_request | 37.33 |
| web_server/reuse_buffer | 37.69 |

## Viterbi Benchmarks

| Benchmark | Time (µs) |
|-----------|-----------|
| short_sentence | 0.71 |
| medium_sentence | 5.65 |
| long_sentence | 28.22 |
| with_lattice_creation | 3.25 |
| lattice_reuse | 0.57 |

## Optimization Targets

Based on the benchmark data, potential optimization areas:

1. **Tokenizer scalability** - Super-linear scaling from 100 to 1000 chars (2x → 50x). The lattice/Viterbi path may have O(n^2) characteristics on longer texts.
2. **Batch processing** - Streaming and collect-all have similar throughput (~1.9 MiB/s), suggesting the bottleneck is tokenization itself, not I/O.
3. **Memory long text** - Processing in chunks (8.3ms) is 3.5x faster than all at once (29.3ms) for long texts, validating the streaming approach.
4. **Lattice reuse** - Reusing lattice (0.57 µs) is 5.7x faster than creating new (3.25 µs).
5. **Matrix lookup** - Already fast (2ns per lookup), not a bottleneck.
6. **Trie search** - Common prefix search at 100ns is fast enough.
