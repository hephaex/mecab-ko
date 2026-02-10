//! Memory profiling benchmarks for MeCab-Ko components.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mecab_ko_profiler::prelude::*;

/// Benchmark memory allocation patterns for dictionary operations.
fn bench_dict_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("dictionary");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("lexicon_load", size), size, |b, &size| {
            b.iter(|| {
                let _guard = MemoryGuard::new(format!("lexicon_{size}"));

                // Simulate lexicon loading
                let data: Vec<Vec<u8>> = (0..size)
                    .map(|i| format!("word_{i}").into_bytes())
                    .collect();

                black_box(data);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("connection_costs", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _guard = MemoryGuard::new(format!("costs_{size}"));

                    // Simulate connection cost matrix
                    let matrix: Vec<i16> = vec![0; size * size];

                    black_box(matrix);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark memory allocation patterns for tokenization.
fn bench_tokenization(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenization");

    let test_texts = [("short", "한국어"),
        ("medium", "한국어 형태소 분석 테스트"),
        (
            "long",
            "한국어 형태소 분석은 자연어 처리의 기본 기술입니다.",
        )];

    for (name, text) in test_texts.iter() {
        group.bench_with_input(BenchmarkId::new("tokenize", name), text, |b, &text| {
            b.iter(|| {
                let _guard = MemoryGuard::new(format!("tokenize_{}", text.len()));

                // Simulate tokenization
                let tokens: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();

                black_box(tokens);
            });
        });

        group.bench_with_input(BenchmarkId::new("lattice", name), text, |b, &text| {
            b.iter(|| {
                let _guard = MemoryGuard::new(format!("lattice_{}", text.len()));

                // Simulate lattice construction
                let nodes: Vec<(usize, String)> = text
                    .chars()
                    .enumerate()
                    .map(|(i, c)| (i, c.to_string()))
                    .collect();

                black_box(nodes);
            });
        });
    }

    group.finish();
}

/// Benchmark memory allocation patterns for Trie structures.
fn bench_trie_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("trie");

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("fst_build", size), size, |b, &size| {
            b.iter(|| {
                let _guard = MemoryGuard::new(format!("fst_{size}"));

                // Simulate FST construction
                let data: Vec<(String, u64)> = (0..size)
                    .map(|i| (format!("key_{i:08}"), i as u64))
                    .collect();

                black_box(data);
            });
        });

        group.bench_with_input(
            BenchmarkId::new("double_array_build", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let _guard = MemoryGuard::new(format!("da_{size}"));

                    // Simulate double-array construction
                    let data: Vec<String> = (0..size).map(|i| format!("key_{i:08}")).collect();

                    black_box(data);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark allocation patterns for different data sizes.
fn bench_allocation_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocation_patterns");

    // Small allocations
    group.bench_function("small_many", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("small_many");

            let data: Vec<Vec<u8>> = (0..1000).map(|_| vec![0u8; 64]).collect();

            black_box(data);
        });
    });

    // Large allocations
    group.bench_function("large_few", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("large_few");

            let data: Vec<Vec<u8>> = (0..10).map(|_| vec![0u8; 64000]).collect();

            black_box(data);
        });
    });

    // Mixed allocations
    group.bench_function("mixed", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("mixed");

            let small: Vec<u8> = vec![0u8; 64];
            let medium: Vec<u8> = vec![0u8; 1024];
            let large: Vec<u8> = vec![0u8; 65536];

            black_box((small, medium, large));
        });
    });

    group.finish();
}

/// Benchmark memory efficiency of different collection types.
fn bench_collection_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("collections");

    let size = 10000;

    group.bench_function("vec_preallocated", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("vec_prealloc");

            let mut data = Vec::with_capacity(size);
            for i in 0..size {
                data.push(i);
            }

            black_box(data);
        });
    });

    group.bench_function("vec_growing", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("vec_growing");

            let mut data = Vec::new();
            for i in 0..size {
                data.push(i);
            }

            black_box(data);
        });
    });

    group.bench_function("string_builder", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("string_builder");

            let mut s = String::new();
            for i in 0..1000 {
                s.push_str(&format!("item_{i} "));
            }

            black_box(s);
        });
    });

    group.finish();
}

/// Benchmark memory profiling overhead.
fn bench_profiling_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("overhead");

    group.bench_function("without_guard", |b| {
        b.iter(|| {
            let data: Vec<u8> = vec![0u8; 1024];
            black_box(data);
        });
    });

    group.bench_function("with_guard", |b| {
        b.iter(|| {
            let _guard = MemoryGuard::new("overhead_test");
            let data: Vec<u8> = vec![0u8; 1024];
            black_box(data);
        });
    });

    group.bench_function("snapshot_only", |b| {
        b.iter(|| {
            let snap = snapshot();
            let data: Vec<u8> = vec![0u8; 1024];
            black_box((snap, data));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_dict_operations,
    bench_tokenization,
    bench_trie_operations,
    bench_allocation_patterns,
    bench_collection_efficiency,
    bench_profiling_overhead,
);

criterion_main!(benches);
