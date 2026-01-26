//! # Viterbi 알고리즘 벤치마크
//!
//! Viterbi 탐색, 연접 비용 계산 등의 성능 벤치마크

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use mecab_ko_core::lattice::{Lattice, NodeBuilder, NodeType};
use mecab_ko_core::viterbi::{ConnectionCost, FixedConnectionCost, SpacePenalty, ViterbiSearcher};

/// 간단한 연접 행렬 생성 (테스트용)
fn create_test_matrix() -> FixedConnectionCost {
    FixedConnectionCost::new(100)
}

/// 테스트용 Lattice 생성
fn create_test_lattice(text: &str) -> Lattice {
    let mut lattice = Lattice::new(text);

    // 간단한 노드 추가
    let char_len = lattice.char_len();
    for pos in 0..char_len {
        for len in 1..=2 {
            if pos + len <= char_len {
                let substr = lattice.substring(pos, pos + len);
                lattice.add_node(
                    NodeBuilder::new(substr.as_ref(), pos, pos + len)
                        .left_id((pos % 50) as u16)
                        .right_id((len % 50) as u16)
                        .word_cost((1000 - pos * 10) as i32)
                        .node_type(NodeType::Known)
                        .feature("NNG,*,T,테스트,*,*,*,*"),
                );
            }
        }
    }

    lattice
}

/// Viterbi 탐색 벤치마크 (기본)
fn bench_viterbi_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("viterbi_search");

    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let texts = vec![
        ("short", "안녕하세요"),
        ("medium", "오늘 날씨가 정말 좋습니다"),
        ("long", "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다"),
    ];

    for (name, text) in texts {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            b.iter(|| {
                let mut lattice = create_test_lattice(black_box(text));
                let path = searcher.search(&mut lattice, &matrix);
                black_box(path)
            })
        });
    }

    group.finish();
}

/// 띄어쓰기 패널티 적용 벤치마크
fn bench_space_penalty(c: &mut Criterion) {
    let mut group = c.benchmark_group("space_penalty");

    let matrix = create_test_matrix();

    let text = "오늘 날씨가 정말 좋습니다";

    // 패널티 없음
    group.bench_function("no_penalty", |b| {
        let searcher = ViterbiSearcher::new();
        b.iter(|| {
            let mut lattice = create_test_lattice(black_box(text));
            let path = searcher.search(&mut lattice, &matrix);
            black_box(path)
        })
    });

    // 한국어 기본 패널티
    group.bench_function("korean_default", |b| {
        let searcher = ViterbiSearcher::new().with_space_penalty(SpacePenalty::korean_default());
        b.iter(|| {
            let mut lattice = create_test_lattice(black_box(text));
            let path = searcher.search(&mut lattice, &matrix);
            black_box(path)
        })
    });

    // 커스텀 패널티
    group.bench_function("custom_penalty", |b| {
        let penalty = SpacePenalty::from_dicrc("1785,5000;1786,5000");
        let searcher = ViterbiSearcher::new().with_space_penalty(penalty);
        b.iter(|| {
            let mut lattice = create_test_lattice(black_box(text));
            let path = searcher.search(&mut lattice, &matrix);
            black_box(path)
        })
    });

    group.finish();
}

/// 대규모 Lattice에 대한 Viterbi 탐색
fn bench_large_lattice_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_lattice_search");

    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다. \
                특히 한국어 형태소 분석은 교착어의 특성상 매우 복잡한 과정을 거치게 됩니다. \
                MeCab은 일본어 형태소 분석기로 시작되었지만, 한국어에도 적용되어 \
                높은 성능과 정확도를 보여주고 있습니다.";

    group.bench_function("200_chars", |b| {
        b.iter(|| {
            let mut lattice = create_test_lattice(black_box(text));
            let path = searcher.search(&mut lattice, &matrix);
            black_box(path)
        })
    });

    group.finish();
}

/// 연접 비용 계산 벤치마크
fn bench_connection_cost(c: &mut Criterion) {
    let mut group = c.benchmark_group("connection_cost");

    let matrix = create_test_matrix();

    group.bench_function("single_lookup", |b| {
        b.iter(|| {
            let cost = matrix.cost(black_box(10), black_box(20));
            black_box(cost)
        })
    });

    group.bench_function("batch_lookups", |b| {
        b.iter(|| {
            let mut total = 0i32;
            for left in 0..50 {
                for right in 0..50 {
                    total = total.saturating_add(matrix.cost(left, right));
                }
            }
            black_box(total)
        })
    });

    group.finish();
}

/// Viterbi 탐색 반복 실행 (캐시 효과 측정)
fn bench_repeated_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_search");

    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();
    let text = "오늘 날씨가 정말 좋습니다";

    group.bench_function("10_iterations", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let mut lattice = create_test_lattice(black_box(text));
                let path = searcher.search(&mut lattice, &matrix);
                black_box(path);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_viterbi_search,
    bench_space_penalty,
    bench_large_lattice_search,
    bench_connection_cost,
    bench_repeated_search,
);

criterion_main!(benches);
