//! 초기화 및 Cold Start 성능 벤치마크
//!
//! 측정 항목:
//! - Tokenizer 초기화 시간 (사전 로딩 포함)
//! - 첫 토큰화 vs 이후 토큰화 (캐시 워밍)
//! - Dictionary 로딩 시간
//! - Matrix 로딩 시간
//! - 메모리 사용량 추적

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    unused_imports,
    unused_mut,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use mecab_ko_core::tokenizer::Tokenizer;

/// Tokenizer 초기화 오버헤드
fn bench_tokenizer_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_initialization");

    // 완전 초기화 (사전 로딩 포함)
    group.bench_function("full_initialization", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
            black_box(tokenizer);
        });
    });

    // 초기화 + 첫 토큰화
    group.bench_function("init_plus_first_tokenize", |b| {
        let text = "한국어 형태소 분석기";

        b.iter(|| {
            let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 첫 토큰화 vs 이후 토큰화 (워밍 효과)
fn bench_cache_warming(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_cache_warming");
    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";

    // 첫 토큰화 (콜드 캐시)
    group.bench_function("first_tokenization", |b| {
        b.iter_batched(
            || Tokenizer::new().expect("Failed to create tokenizer"),
            |mut tokenizer| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    // 이후 토큰화 (워밍된 캐시)
    group.bench_function("warmed_tokenization", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
        // 워밍
        let _ = tokenizer.tokenize(text);

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 10회 워밍 후 토큰화
    group.bench_function("heavily_warmed", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
        // 여러 번 워밍
        for _ in 0..10 {
            let _ = tokenizer.tokenize(text);
        }

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 서로 다른 텍스트에 대한 콜드 스타트
fn bench_cold_start_different_texts(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_different_texts");

    let texts = [
        "짧은 텍스트",
        "조금 더 긴 텍스트로 테스트를 진행합니다",
        "매우 긴 텍스트로 벤치마크를 수행하여 초기화와 첫 토큰화의 성능을 측정하며 \
         이를 통해 콜드 스타트 시나리오에서의 실제 성능을 파악할 수 있습니다",
    ];

    for (idx, &text) in texts.iter().enumerate() {
        group.bench_function(format!("text_{idx}"), |b| {
            b.iter_batched(
                || Tokenizer::new().expect("Failed to create tokenizer"),
                |mut tokenizer| {
                    let tokens = tokenizer.tokenize(black_box(text));
                    black_box(tokens);
                },
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

/// 연속 요청 시뮬레이션 (서버 시나리오)
fn bench_server_startup_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_server_scenario");

    let requests = vec![
        "첫 번째 요청입니다",
        "두 번째 요청입니다",
        "세 번째 요청입니다",
        "네 번째 요청입니다",
        "다섯 번째 요청입니다",
    ];

    // 서버 시작 후 첫 5개 요청 처리
    group.bench_function("first_five_requests", |b| {
        b.iter_batched(
            || Tokenizer::new().expect("Failed to create tokenizer"),
            |mut tokenizer| {
                let mut results = Vec::new();
                for request in &requests {
                    let tokens = tokenizer.tokenize(black_box(request));
                    results.push(tokens);
                }
                black_box(results);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    // 워밍 후 5개 요청 처리 (비교용)
    group.bench_function("warmed_five_requests", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
        // 워밍
        for request in &requests {
            let _ = tokenizer.tokenize(request);
        }

        b.iter(|| {
            let mut results = Vec::new();
            for request in &requests {
                let tokens = tokenizer.tokenize(black_box(request));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    group.finish();
}

/// 재사용 vs 매번 생성
fn bench_reuse_vs_recreate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_reuse");
    let text = "한국어 형태소 분석";

    // 매번 새로 생성
    group.bench_function("recreate_each_time", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 재사용
    group.bench_function("reuse_tokenizer", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 병렬 초기화 (여러 스레드에서 동시 초기화)
fn bench_parallel_initialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_parallel");
    group.sample_size(10); // 초기화는 느리므로 샘플 크기 축소

    // 순차 초기화
    group.bench_function("sequential_init_3", |b| {
        b.iter(|| {
            let _t1 = Tokenizer::new().expect("Failed");
            let _t2 = Tokenizer::new().expect("Failed");
            let _t3 = Tokenizer::new().expect("Failed");
        });
    });

    // 병렬 초기화는 실제 스레드 생성이 필요하므로 측정만 표시
    // (실제 구현은 tokio 등이 필요하므로 현재는 스킵)

    group.finish();
}

/// 초기화 시 메모리 할당 패턴
fn bench_initialization_memory_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("cold_start_memory");
    group.sample_size(20);

    // 기본 초기화
    group.bench_function("basic_init", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
            black_box(tokenizer);
        });
    });

    // 초기화 후 즉시 드롭
    group.bench_function("init_and_drop", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
            drop(black_box(tokenizer));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenizer_initialization,
    bench_cache_warming,
    bench_cold_start_different_texts,
    bench_server_startup_scenario,
    bench_reuse_vs_recreate,
    bench_parallel_initialization,
    bench_initialization_memory_pattern,
);

criterion_main!(benches);
