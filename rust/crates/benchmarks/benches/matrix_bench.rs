//! Matrix 연접 비용 조회 성능 벤치마크
//!
//! 측정 항목:
//! - 랜덤 조회 성능
//! - 순차 조회 성능
//! - 캐시 효율성
//! - 다양한 행렬 크기에서의 성능

#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_dict::matrix::{DenseMatrix, Matrix};
use rand::Rng;

/// 소형 연접 비용 행렬 생성 (100x100)
fn create_small_matrix() -> DenseMatrix {
    let size = 100;
    let mut matrix = DenseMatrix::new(size, size, 0);

    // 실제 학습 데이터와 유사한 패턴으로 비용 설정
    let mut rng = rand::thread_rng();
    for left_id in 0..size {
        for right_id in 0..size {
            // 대부분 낮은 비용, 일부 높은 비용
            let cost = if rng.gen_bool(0.8) {
                rng.gen_range(0..500)
            } else {
                rng.gen_range(1000..5000)
            };
            matrix.set(right_id as u16, left_id as u16, cost);
        }
    }

    matrix
}

/// 중형 연접 비용 행렬 생성 (1000x1000)
fn create_medium_matrix() -> DenseMatrix {
    let size = 1000;
    let mut matrix = DenseMatrix::new(size, size, 100);

    let mut rng = rand::thread_rng();
    // 전체를 설정하면 너무 오래 걸리므로 샘플링
    for _ in 0..10000 {
        let left_id = rng.gen_range(0..size) as u16;
        let right_id = rng.gen_range(0..size) as u16;
        let cost = rng.gen_range(0..3000);
        matrix.set(right_id, left_id, cost);
    }

    matrix
}

/// 대형 연접 비용 행렬 생성 (2000x2000) - mecab-ko-dic 실제 크기에 근접
fn create_large_matrix() -> DenseMatrix {
    let left_size = 2000;
    let right_size = 2000;
    let mut matrix = DenseMatrix::new(left_size, right_size, 100);

    let mut rng = rand::thread_rng();
    // 샘플링으로 일부만 설정
    for _ in 0..50000 {
        let left_id = rng.gen_range(0..left_size) as u16;
        let right_id = rng.gen_range(0..right_size) as u16;
        let cost = rng.gen_range(0..5000);
        matrix.set(right_id, left_id, cost);
    }

    matrix
}

/// 단일 조회 성능
fn bench_single_lookup(c: &mut Criterion) {
    let matrix = create_medium_matrix();

    let mut group = c.benchmark_group("matrix_single_lookup");
    group.throughput(Throughput::Elements(1));

    // 연속된 ID 조회 (캐시 친화적)
    group.bench_function("sequential", |b| {
        let mut idx = 0u16;
        b.iter(|| {
            let cost = matrix.get(black_box(idx % 100), black_box((idx + 1) % 100));
            idx = (idx + 1) % 100;
            black_box(cost);
        });
    });

    // 랜덤 ID 조회 (캐시 비친화적)
    group.bench_function("random", |b| {
        let mut rng = rand::thread_rng();
        b.iter(|| {
            let right_id = rng.gen_range(0..1000);
            let left_id = rng.gen_range(0..1000);
            let cost = matrix.get(black_box(right_id), black_box(left_id));
            black_box(cost);
        });
    });

    // 고정 ID 조회 (최대 캐시 효과)
    group.bench_function("fixed", |b| {
        b.iter(|| {
            let cost = matrix.get(black_box(100), black_box(200));
            black_box(cost);
        });
    });

    group.finish();
}

/// 배치 조회 성능
fn bench_batch_lookup(c: &mut Criterion) {
    let matrix = create_medium_matrix();

    let mut group = c.benchmark_group("matrix_batch_lookup");

    for batch_size in &[10, 100, 1000] {
        group.throughput(Throughput::Elements(*batch_size));

        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &size| {
                // 미리 조회할 ID 쌍 생성
                let mut rng = rand::thread_rng();
                let queries: Vec<(u16, u16)> = (0..size)
                    .map(|_| (rng.gen_range(0..1000), rng.gen_range(0..1000)))
                    .collect();

                b.iter(|| {
                    let mut total = 0i64;
                    for &(right_id, left_id) in &queries {
                        total += i64::from(matrix.get(right_id, left_id));
                    }
                    black_box(total);
                });
            },
        );
    }

    group.finish();
}

/// Viterbi 알고리즘 시뮬레이션 패턴
fn bench_viterbi_access_pattern(c: &mut Criterion) {
    let matrix = create_medium_matrix();

    let mut group = c.benchmark_group("matrix_viterbi_pattern");

    // Viterbi에서는 이전 노드의 right_id와 현재 노드의 left_id를 조회
    // 각 노드에 대해 여러 이전 노드를 검사
    group.bench_function("node_transition", |b| {
        // 현재 노드 left_id
        let current_left_ids = vec![100u16, 150, 200, 250, 300];
        // 이전 노드 right_id 후보들
        let prev_right_ids = vec![50u16, 75, 100, 125, 150, 175, 200];

        b.iter(|| {
            let mut min_cost = i32::MAX;
            for &left_id in &current_left_ids {
                for &right_id in &prev_right_ids {
                    let cost = matrix.get(black_box(right_id), black_box(left_id));
                    min_cost = min_cost.min(cost);
                }
            }
            black_box(min_cost);
        });
    });

    // 전체 경로 계산 시뮬레이션
    group.bench_function("path_calculation", |b| {
        // 10개 노드, 각 위치마다 5개 후보
        let positions = 10;
        let candidates_per_pos = 5;

        b.iter(|| {
            let mut prev_ids = vec![0u16, 10, 20, 30, 40]; // 초기 노드들
            let mut total_cost = 0i64;

            for _pos in 0..positions {
                let current_ids: Vec<u16> =
                    (0..candidates_per_pos).map(|i| (i * 50) as u16).collect();

                // 각 현재 노드에 대해 이전 노드들과의 연접 비용 계산
                for &curr in &current_ids {
                    for &prev in &prev_ids {
                        let cost = matrix.get(prev, curr);
                        total_cost += i64::from(cost);
                    }
                }

                prev_ids = current_ids;
            }

            black_box(total_cost);
        });
    });

    group.finish();
}

/// 다양한 크기의 행렬 성능 비교
fn bench_different_sizes(c: &mut Criterion) {
    let small = create_small_matrix();
    let medium = create_medium_matrix();
    let large = create_large_matrix();

    let mut group = c.benchmark_group("matrix_size_comparison");
    group.throughput(Throughput::Elements(100));

    group.bench_function("small_100x100", |b| {
        let mut rng = rand::thread_rng();
        b.iter(|| {
            let mut total = 0i64;
            for _ in 0..100 {
                let right_id = rng.gen_range(0..100);
                let left_id = rng.gen_range(0..100);
                total += i64::from(small.get(right_id, left_id));
            }
            black_box(total);
        });
    });

    group.bench_function("medium_1000x1000", |b| {
        let mut rng = rand::thread_rng();
        b.iter(|| {
            let mut total = 0i64;
            for _ in 0..100 {
                let right_id = rng.gen_range(0..1000);
                let left_id = rng.gen_range(0..1000);
                total += i64::from(medium.get(right_id, left_id));
            }
            black_box(total);
        });
    });

    group.bench_function("large_2000x2000", |b| {
        let mut rng = rand::thread_rng();
        b.iter(|| {
            let mut total = 0i64;
            for _ in 0..100 {
                let right_id = rng.gen_range(0..2000);
                let left_id = rng.gen_range(0..2000);
                total += i64::from(large.get(right_id, left_id));
            }
            black_box(total);
        });
    });

    group.finish();
}

/// 캐시 지역성 테스트
fn bench_cache_locality(c: &mut Criterion) {
    let matrix = create_medium_matrix();

    let mut group = c.benchmark_group("matrix_cache_locality");
    group.throughput(Throughput::Elements(1000));

    // 행 우선 순회 (캐시 친화적)
    group.bench_function("row_major", |b| {
        b.iter(|| {
            let mut total = 0i64;
            for left_id in 0..10u16 {
                for right_id in 0..100u16 {
                    total += i64::from(matrix.get(right_id, left_id));
                }
            }
            black_box(total);
        });
    });

    // 열 우선 순회 (상대적으로 캐시 비친화적)
    group.bench_function("column_major", |b| {
        b.iter(|| {
            let mut total = 0i64;
            for right_id in 0..100u16 {
                for left_id in 0..10u16 {
                    total += i64::from(matrix.get(right_id, left_id));
                }
            }
            black_box(total);
        });
    });

    // 스트라이드 접근
    group.bench_function("strided", |b| {
        b.iter(|| {
            let mut total = 0i64;
            for i in (0..1000).step_by(10) {
                let right_id = (i % 1000) as u16;
                let left_id = ((i / 10) % 1000) as u16;
                total += i64::from(matrix.get(right_id, left_id));
            }
            black_box(total);
        });
    });

    group.finish();
}

/// 메모리 사용량 측정
fn bench_memory_footprint(c: &mut Criterion) {
    let mut group = c.benchmark_group("matrix_memory");

    for &size in &[100, 500, 1000, 2000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let matrix = DenseMatrix::new(size, size, 100);

                // 메모리 크기 계산
                let memory_bytes = size * size * std::mem::size_of::<i16>();
                black_box((matrix, memory_bytes));
            });
        });
    }

    group.finish();
}

/// 실제 mecab-ko-dic 사용 패턴 시뮬레이션
fn bench_realistic_workload(c: &mut Criterion) {
    let matrix = create_large_matrix();

    let mut group = c.benchmark_group("matrix_realistic_workload");

    // 문장 분석 시뮬레이션: 10개 위치, 각 위치마다 평균 3개 후보
    group.bench_function("sentence_10words", |b| {
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let mut total_cost = 0i64;

            // 각 위치에서의 노드 후보
            for _pos in 0..10 {
                let num_candidates = rng.gen_range(1..=5);
                let prev_right_id = rng.gen_range(0..2000);

                for _ in 0..num_candidates {
                    let curr_left_id = rng.gen_range(0..2000);
                    let cost = matrix.get(prev_right_id, curr_left_id);
                    total_cost += i64::from(cost);
                }
            }

            black_box(total_cost);
        });
    });

    // 긴 문장 분석
    group.bench_function("sentence_50words", |b| {
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let mut total_cost = 0i64;

            for _pos in 0..50 {
                let num_candidates = rng.gen_range(1..=4);
                let prev_right_id = rng.gen_range(0..2000);

                for _ in 0..num_candidates {
                    let curr_left_id = rng.gen_range(0..2000);
                    let cost = matrix.get(prev_right_id, curr_left_id);
                    total_cost += i64::from(cost);
                }
            }

            black_box(total_cost);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_lookup,
    bench_batch_lookup,
    bench_viterbi_access_pattern,
    bench_different_sizes,
    bench_cache_locality,
    bench_memory_footprint,
    bench_realistic_workload,
);

criterion_main!(benches);
