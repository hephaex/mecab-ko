//! # Lattice 구축 벤치마크
//!
//! Lattice 생성, 노드 추가, 검색 등의 성능 벤치마크

#![allow(
    clippy::semicolon_if_nothing_returned,
    clippy::useless_asref,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mecab_ko_core::lattice::{Lattice, NodeBuilder, NodeType};

/// Lattice 생성 벤치마크
fn bench_lattice_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lattice_creation");

    let texts = vec![
        ("short", "안녕하세요"),
        ("medium", "오늘 날씨가 정말 좋습니다"),
        (
            "long",
            "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다",
        ),
    ];

    for (name, text) in texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            b.iter(|| {
                let lattice = Lattice::new(black_box(text));
                black_box(lattice)
            })
        });
    }

    group.finish();
}

/// Lattice 노드 추가 벤치마크
fn bench_node_addition(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_addition");

    let text = "안녕하세요 반갑습니다";
    let mut lattice = Lattice::new(text);

    group.bench_function("single_node", |b| {
        b.iter(|| {
            lattice.add_node(
                NodeBuilder::new("안녕", 0, 2)
                    .left_id(1)
                    .right_id(1)
                    .word_cost(1000)
                    .node_type(NodeType::Known)
                    .feature("NNG,*,T,안녕,*,*,*,*"),
            );
        })
    });

    group.bench_function("multiple_nodes", |b| {
        b.iter(|| {
            for i in 0..10 {
                lattice.add_node(
                    NodeBuilder::new("테스트", i, i + 3)
                        .left_id(1)
                        .right_id(1)
                        .word_cost(1000)
                        .node_type(NodeType::Known)
                        .feature("NNG,*,T,테스트,*,*,*,*"),
                );
            }
        })
    });

    group.finish();
}

/// Lattice 리셋 벤치마크 (재사용 성능)
fn bench_lattice_reset(c: &mut Criterion) {
    let mut group = c.benchmark_group("lattice_reset");

    let mut lattice = Lattice::new("초기 텍스트");

    // 노드를 많이 추가
    for i in 0..100 {
        lattice.add_node(
            NodeBuilder::new("테스트", i % 10, (i % 10) + 1)
                .left_id(1)
                .right_id(1)
                .word_cost(1000)
                .node_type(NodeType::Known)
                .feature("NNG,*,T,테스트,*,*,*,*"),
        );
    }

    group.bench_function("reset", |b| {
        b.iter(|| {
            lattice.reset(black_box("새로운 텍스트입니다"));
        })
    });

    group.finish();
}

/// Lattice 노드 검색 벤치마크
fn bench_node_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_lookup");

    let text = "안녕하세요 반갑습니다";
    let mut lattice = Lattice::new(text);

    // 여러 노드 추가
    for i in 0..20 {
        lattice.add_node(
            NodeBuilder::new("테스트", i % 10, (i % 10) + 1)
                .left_id(1)
                .right_id(1)
                .word_cost(1000)
                .node_type(NodeType::Known)
                .feature("NNG,*,T,테스트,*,*,*,*"),
        );
    }

    group.bench_function("single_lookup", |b| {
        b.iter(|| {
            let node = lattice.node(black_box(5));
            black_box(node)
        })
    });

    group.bench_function("multiple_lookups", |b| {
        b.iter(|| {
            for id in 0..20 {
                let node = lattice.node(black_box(id));
                black_box(node);
            }
        })
    });

    group.finish();
}

/// Lattice substring 추출 벤치마크
fn bench_substring(c: &mut Criterion) {
    let mut group = c.benchmark_group("substring");

    let text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다";
    let lattice = Lattice::new(text);

    group.bench_function("short_substring", |b| {
        b.iter(|| {
            let substr = lattice.substring(black_box(0), black_box(5));
            black_box(substr)
        })
    });

    group.bench_function("medium_substring", |b| {
        b.iter(|| {
            let substr = lattice.substring(black_box(0), black_box(20));
            black_box(substr)
        })
    });

    group.bench_function("full_substring", |b| {
        b.iter(|| {
            let substr = lattice.substring(black_box(0), black_box(lattice.char_len()));
            black_box(substr)
        })
    });

    group.finish();
}

/// Lattice 통계 정보 계산 벤치마크
fn bench_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("stats");

    let text = "안녕하세요 반갑습니다";
    let mut lattice = Lattice::new(text);

    // 많은 노드 추가
    for i in 0..100 {
        lattice.add_node(
            NodeBuilder::new("테스트", i % 10, (i % 10) + 1)
                .left_id(1)
                .right_id(1)
                .word_cost(1000)
                .node_type(NodeType::Known)
                .feature("NNG,*,T,테스트,*,*,*,*"),
        );
    }

    group.bench_function("stats", |b| {
        b.iter(|| {
            let stats = lattice.stats();
            black_box(stats)
        })
    });

    group.finish();
}

/// 대규모 Lattice 구축 시나리오
fn bench_large_lattice(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_lattice");

    let text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다. \
                특히 한국어 형태소 분석은 교착어의 특성상 매우 복잡한 과정을 거치게 됩니다.";

    group.bench_function("create_and_populate", |b| {
        b.iter(|| {
            let mut lattice = Lattice::new(black_box(text));

            // 각 위치에 여러 노드 추가 (사전 검색 시뮬레이션)
            for pos in 0..lattice.char_len() {
                for len in 1..=3 {
                    if pos + len <= lattice.char_len() {
                        let substr = lattice.substring(pos, pos + len);
                        lattice.add_node(
                            NodeBuilder::new(substr.as_ref(), pos, pos + len)
                                .left_id((pos % 10) as u16)
                                .right_id((len % 10) as u16)
                                .word_cost((1000 + pos * 10) as i32)
                                .node_type(NodeType::Known)
                                .feature("NNG,*,T,테스트,*,*,*,*"),
                        );
                    }
                }
            }

            black_box(lattice)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_lattice_creation,
    bench_node_addition,
    bench_lattice_reset,
    bench_node_lookup,
    bench_substring,
    bench_stats,
    bench_large_lattice,
);

criterion_main!(benches);
