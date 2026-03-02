//! Viterbi 알고리즘 성능 벤치마크
//!
//! 측정 항목:
//! - Forward pass 성능
//! - Backward pass 성능
//! - 전체 경로 탐색 성능
//! - 노드 수에 따른 확장성
//! - N-best 성능

#![allow(clippy::cast_possible_truncation, missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::lattice::{Lattice, NodeBuilder};
use mecab_ko_core::nbest::ImprovedNbestSearcher;
use mecab_ko_core::viterbi::{NbestSearcher, SpacePenalty, ViterbiSearcher, ZeroConnectionCost};
use mecab_ko_dict::matrix::DenseMatrix;
use rand::Rng;

/// 테스트용 연접 비용 행렬
fn create_test_matrix() -> DenseMatrix {
    let size = 100;
    let mut matrix = DenseMatrix::new(size, size, 100);

    let mut rng = rand::thread_rng();
    for left_id in 0..size {
        for right_id in 0..size {
            let cost = rng.gen_range(0..1000);
            matrix.set(right_id as u16, left_id as u16, cost);
        }
    }

    matrix
}

/// 소형 Lattice 생성 (5개 노드, 단일 경로)
fn create_small_lattice() -> Lattice {
    let mut lattice = Lattice::new("ABCDE");

    // 단순 선형 경로: A -> B -> C -> D -> E
    lattice.add_node(
        NodeBuilder::new("A", 0, 1)
            .left_id(1)
            .right_id(1)
            .word_cost(100),
    );

    lattice.add_node(
        NodeBuilder::new("B", 1, 2)
            .left_id(2)
            .right_id(2)
            .word_cost(200),
    );

    lattice.add_node(
        NodeBuilder::new("C", 2, 3)
            .left_id(3)
            .right_id(3)
            .word_cost(150),
    );

    lattice.add_node(
        NodeBuilder::new("D", 3, 4)
            .left_id(4)
            .right_id(4)
            .word_cost(180),
    );

    lattice.add_node(
        NodeBuilder::new("E", 4, 5)
            .left_id(5)
            .right_id(5)
            .word_cost(120),
    );

    lattice
}

/// 중형 Lattice 생성 (다중 경로)
fn create_medium_lattice() -> Lattice {
    let mut lattice = Lattice::new("아버지가방에");

    // 위치 0-3: "아버지" vs "아버"
    lattice.add_node(
        NodeBuilder::new("아버지", 0, 3)
            .left_id(10)
            .right_id(10)
            .word_cost(1000),
    );

    lattice.add_node(
        NodeBuilder::new("아버", 0, 2)
            .left_id(11)
            .right_id(11)
            .word_cost(3000),
    );

    // 위치 2-3: "지"
    lattice.add_node(
        NodeBuilder::new("지", 2, 3)
            .left_id(12)
            .right_id(12)
            .word_cost(500),
    );

    // 위치 3-4: "가" vs "가방"
    lattice.add_node(
        NodeBuilder::new("가", 3, 4)
            .left_id(20)
            .right_id(20)
            .word_cost(500),
    );

    lattice.add_node(
        NodeBuilder::new("가방", 3, 5)
            .left_id(21)
            .right_id(21)
            .word_cost(800),
    );

    // 위치 4-5: "방"
    lattice.add_node(
        NodeBuilder::new("방", 4, 5)
            .left_id(22)
            .right_id(22)
            .word_cost(600),
    );

    // 위치 5-6: "에"
    lattice.add_node(
        NodeBuilder::new("에", 5, 6)
            .left_id(30)
            .right_id(30)
            .word_cost(300),
    );

    lattice
}

/// 대형 Lattice 생성 (복잡한 다중 경로)
fn create_large_lattice() -> Lattice {
    let text = "한국어형태소분석기는자연어처리의핵심기술입니다";
    let mut lattice = Lattice::new(text);
    let mut rng = rand::thread_rng();

    // 각 문자 위치에서 1~3개의 노드 생성
    let char_count = text.chars().count();
    for start_pos in 0..char_count {
        let num_nodes = rng.gen_range(1..=3);

        for _ in 0..num_nodes {
            let length = rng.gen_range(1..=(char_count - start_pos).min(5));
            let end_pos = start_pos + length;

            // 표면형 추출
            let surface: String = text.chars().skip(start_pos).take(length).collect();

            lattice.add_node(
                NodeBuilder::new(&surface, start_pos, end_pos)
                    .left_id(rng.gen_range(0..100))
                    .right_id(rng.gen_range(0..100))
                    .word_cost(rng.gen_range(100..2000)),
            );
        }
    }

    lattice
}

/// 기본 Viterbi 검색 성능
fn bench_viterbi_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("viterbi_search");

    // 소형 lattice - 제로 비용
    group.bench_function("small_zero_cost", |b| {
        let conn_cost = ZeroConnectionCost;
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let mut lattice = create_small_lattice();
            let path = searcher.search(black_box(&mut lattice), &conn_cost);
            black_box(path);
        });
    });

    // 소형 lattice - 실제 비용 행렬
    group.bench_function("small_with_matrix", |b| {
        let matrix = create_test_matrix();
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let mut lattice = create_small_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 중형 lattice
    group.bench_function("medium", |b| {
        let matrix = create_test_matrix();
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 대형 lattice
    group.bench_function("large", |b| {
        let matrix = create_test_matrix();
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let mut lattice = create_large_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

/// 띄어쓰기 패널티 오버헤드
fn bench_space_penalty(c: &mut Criterion) {
    let mut lattice = create_medium_lattice();
    let matrix = create_test_matrix();

    let mut group = c.benchmark_group("viterbi_space_penalty");

    // 패널티 없음
    group.bench_function("no_penalty", |b| {
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 기본 한국어 패널티
    group.bench_function("korean_default", |b| {
        let penalty = SpacePenalty::korean_default();
        let searcher = ViterbiSearcher::new().with_space_penalty(penalty);

        b.iter(|| {
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 커스텀 패널티
    group.bench_function("custom_penalty", |b| {
        let mut penalty = SpacePenalty::new();
        for i in 0..50 {
            penalty.add(i, 5000);
        }
        let searcher = ViterbiSearcher::new().with_space_penalty(penalty);

        b.iter(|| {
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

/// N-best 경로 탐색 성능
fn bench_nbest_search(c: &mut Criterion) {
    let matrix = create_test_matrix();

    let mut group = c.benchmark_group("viterbi_nbest");

    for n in &[1, 3, 5, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &n| {
            let searcher = NbestSearcher::new(n);

            b.iter(|| {
                let mut lattice = create_medium_lattice();
                let results = searcher.search(black_box(&mut lattice), &matrix);
                black_box(results);
            });
        });
    }

    group.finish();
}

/// 개선된 N-best 탐색 성능 (ImprovedNbestSearcher)
fn bench_improved_nbest_search(c: &mut Criterion) {
    let matrix = create_test_matrix();

    let mut group = c.benchmark_group("viterbi_improved_nbest");

    for n in &[1, 3, 5, 10] {
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &n| {
            let searcher = ImprovedNbestSearcher::new(n);

            b.iter(|| {
                let mut lattice = create_medium_lattice();
                let results = searcher.search(black_box(&mut lattice), &matrix);
                black_box(results);
            });
        });
    }

    group.finish();
}

/// N-best 구현 비교 (기존 vs 개선)
fn bench_nbest_comparison(c: &mut Criterion) {
    let matrix = create_test_matrix();

    let mut group = c.benchmark_group("viterbi_nbest_comparison");

    // N=5 기준 비교
    let n = 5;

    // 기존 구현
    group.bench_function("legacy_n5", |b| {
        let searcher = NbestSearcher::new(n);

        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let results = searcher.search(black_box(&mut lattice), &matrix);
            black_box(results);
        });
    });

    // 개선된 구현
    group.bench_function("improved_n5", |b| {
        let searcher = ImprovedNbestSearcher::new(n);

        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let results = searcher.search(black_box(&mut lattice), &matrix);
            black_box(results);
        });
    });

    // 대형 lattice에서 N=5 비교
    group.bench_function("legacy_large_n5", |b| {
        let searcher = NbestSearcher::new(n);

        b.iter(|| {
            let mut lattice = create_large_lattice();
            let results = searcher.search(black_box(&mut lattice), &matrix);
            black_box(results);
        });
    });

    group.bench_function("improved_large_n5", |b| {
        let searcher = ImprovedNbestSearcher::new(n);

        b.iter(|| {
            let mut lattice = create_large_lattice();
            let results = searcher.search(black_box(&mut lattice), &matrix);
            black_box(results);
        });
    });

    group.finish();
}

/// 노드 수에 따른 확장성
fn bench_scalability_by_nodes(c: &mut Criterion) {
    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let mut group = c.benchmark_group("viterbi_scalability");

    for num_positions in &[5, 10, 20, 50] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_positions),
            num_positions,
            |b, &positions| {
                b.iter(|| {
                    // positions 길이의 텍스트로 lattice 생성
                    let text = "가".repeat(positions);
                    let mut lattice = Lattice::new(&text);
                    let mut rng = rand::thread_rng();

                    // 각 위치에서 2-3개 노드 추가
                    for start in 0..positions {
                        for _ in 0..2 {
                            let length = rng.gen_range(1..=(positions - start).min(3));
                            let surface: String = "가".repeat(length);

                            lattice.add_node(
                                NodeBuilder::new(&surface, start, start + length)
                                    .left_id(rng.gen_range(0..100))
                                    .right_id(rng.gen_range(0..100))
                                    .word_cost(rng.gen_range(100..1000)),
                            );
                        }
                    }

                    let path = searcher.search(black_box(&mut lattice), &matrix);
                    black_box(path);
                });
            },
        );
    }

    group.finish();
}

/// 경로 복잡도에 따른 성능
fn bench_path_complexity(c: &mut Criterion) {
    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let mut group = c.benchmark_group("viterbi_path_complexity");

    // 단일 경로 (복잡도: 낮음)
    group.bench_function("single_path", |b| {
        b.iter(|| {
            let mut lattice = create_small_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 이중 경로 (복잡도: 중간)
    group.bench_function("dual_path", |b| {
        b.iter(|| {
            let mut lattice = Lattice::new("ABC");

            // 경로 1: A -> B -> C
            lattice.add_node(
                NodeBuilder::new("A", 0, 1)
                    .left_id(1)
                    .right_id(1)
                    .word_cost(100),
            );
            lattice.add_node(
                NodeBuilder::new("B", 1, 2)
                    .left_id(2)
                    .right_id(2)
                    .word_cost(200),
            );

            // 경로 2: AB -> C
            lattice.add_node(
                NodeBuilder::new("AB", 0, 2)
                    .left_id(3)
                    .right_id(3)
                    .word_cost(250),
            );

            lattice.add_node(
                NodeBuilder::new("C", 2, 3)
                    .left_id(4)
                    .right_id(4)
                    .word_cost(150),
            );

            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 다중 경로 (복잡도: 높음)
    group.bench_function("multi_path", |b| {
        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

/// Forward/Backward pass 분리 측정
fn bench_forward_backward_separate(c: &mut Criterion) {
    let matrix = create_test_matrix();

    let mut group = c.benchmark_group("viterbi_passes");

    // Full search (forward + backward)
    group.bench_function("full_search", |b| {
        let searcher = ViterbiSearcher::new();

        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

/// 실제 문장 패턴 시뮬레이션
fn bench_realistic_sentences(c: &mut Criterion) {
    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let mut group = c.benchmark_group("viterbi_realistic");
    group.throughput(Throughput::Elements(1));

    // 짧은 문장 (5-10 음절)
    group.bench_function("short_sentence", |b| {
        b.iter(|| {
            let mut lattice = create_small_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 중간 문장 (10-20 음절)
    group.bench_function("medium_sentence", |b| {
        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // 긴 문장 (50+ 음절)
    group.bench_function("long_sentence", |b| {
        b.iter(|| {
            let mut lattice = create_large_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

/// 메모리 할당 오버헤드
fn bench_memory_overhead(c: &mut Criterion) {
    let matrix = create_test_matrix();
    let searcher = ViterbiSearcher::new();

    let mut group = c.benchmark_group("viterbi_memory");

    // Lattice 생성 포함
    group.bench_function("with_lattice_creation", |b| {
        b.iter(|| {
            let mut lattice = create_medium_lattice();
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    // Lattice 재사용
    group.bench_function("lattice_reuse", |b| {
        let mut lattice = create_medium_lattice();

        b.iter(|| {
            // 동일 lattice 재탐색 (실제로는 비추천, 벤치마크용)
            let path = searcher.search(black_box(&mut lattice), &matrix);
            black_box(path);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_viterbi_search,
    bench_space_penalty,
    bench_nbest_search,
    bench_improved_nbest_search,
    bench_nbest_comparison,
    bench_scalability_by_nodes,
    bench_path_complexity,
    bench_forward_backward_separate,
    bench_realistic_sentences,
    bench_memory_overhead,
);

criterion_main!(benches);
