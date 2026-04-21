//! 배치 토큰화 성능 벤치마크
//!
//! 측정 항목:
//! - 다양한 배치 크기에서의 처리량
//! - 병렬 처리 효율성
//! - 메모리 효율성
//! - 순차 vs 병렬 처리 비교

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::useless_vec,
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::explicit_iter_loop,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::batch::BatchTokenizer;
use mecab_ko_core::tokenizer::Tokenizer;

/// 테스트용 텍스트 생성
fn generate_texts(count: usize) -> Vec<String> {
    let templates = [
        "한국어 형태소 분석기",
        "자연어 처리는 인공지능의 중요한 분야입니다",
        "오늘은 날씨가 맑고 화창합니다",
        "서울은 대한민국의 수도입니다",
        "형태소 분석 결과는 품사 태깅을 포함합니다",
        "프로그래밍 언어는 다양한 용도로 사용됩니다",
        "데이터 과학은 현대 사회의 핵심 기술입니다",
        "머신러닝 모델은 대량의 데이터를 필요로 합니다",
    ];

    (0..count)
        .map(|i| templates[i % templates.len()].to_string())
        .collect()
}

/// 작은 배치 크기 성능
fn bench_small_batches(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_small");

    for size in [1, 5, 10, 20, 50].iter() {
        let texts = generate_texts(*size);
        let total_bytes: usize = texts.iter().map(|t| t.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &texts, |b, texts| {
            b.iter(|| {
                let mut results = Vec::new();
                for text in texts {
                    let tokens = tokenizer.tokenize(black_box(text));
                    results.push(tokens);
                }
                black_box(results);
            });
        });
    }

    group.finish();
}

/// 중간 배치 크기 성능
fn bench_medium_batches(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_medium");

    for size in [100, 200, 500].iter() {
        let texts = generate_texts(*size);
        let total_bytes: usize = texts.iter().map(|t| t.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &texts, |b, texts| {
            b.iter(|| {
                let mut results = Vec::new();
                for text in texts {
                    let tokens = tokenizer.tokenize(black_box(text));
                    results.push(tokens);
                }
                black_box(results);
            });
        });
    }

    group.finish();
}

/// 대용량 배치 처리
fn bench_large_batches(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_large");
    group.sample_size(10); // 대용량은 샘플 크기 축소

    for size in [1000, 2000, 5000].iter() {
        let texts = generate_texts(*size);
        let total_bytes: usize = texts.iter().map(|t| t.len()).sum();

        group.throughput(Throughput::Bytes(total_bytes as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &texts, |b, texts| {
            b.iter(|| {
                let mut results = Vec::new();
                for text in texts {
                    let tokens = tokenizer.tokenize(black_box(text));
                    results.push(tokens);
                }
                black_box(results);
            });
        });
    }

    group.finish();
}

/// 배치 처리 - 다양한 텍스트 길이 혼합
fn bench_mixed_length_batch(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_mixed_length");

    // 짧은 텍스트와 긴 텍스트 혼합
    let short = "짧은 텍스트";
    let medium = "중간 길이의 텍스트로 형태소 분석을 수행합니다";
    let long = "이것은 매우 긴 텍스트입니다. \
                한국어 형태소 분석기는 다양한 길이의 텍스트를 처리할 수 있어야 하며, \
                배치 처리 시 서로 다른 길이의 텍스트가 혼합되어 있을 때의 성능도 중요합니다. \
                이 벤치마크는 그러한 시나리오를 시뮬레이션합니다.";

    let mut mixed_texts = Vec::new();
    for i in 0..100 {
        match i % 3 {
            0 => mixed_texts.push(short.to_string()),
            1 => mixed_texts.push(medium.to_string()),
            _ => mixed_texts.push(long.to_string()),
        }
    }

    let total_bytes: usize = mixed_texts.iter().map(|t| t.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("mixed_100", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for text in &mixed_texts {
                let tokens = tokenizer.tokenize(black_box(text));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    group.finish();
}

/// 처리량 측정 (texts/sec)
fn bench_throughput_metrics(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_throughput");

    let batch_size = 1000;
    let texts = generate_texts(batch_size);

    group.throughput(Throughput::Elements(batch_size as u64));
    group.bench_function("texts_per_second", |b| {
        b.iter(|| {
            let mut count = 0;
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                count += tokens.len();
            }
            black_box(count);
        });
    });

    group.finish();
}

/// 메모리 효율성 - 배치 크기에 따른 메모리 사용
fn bench_batch_memory_efficiency(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_memory");

    for size in [10, 100, 1000].iter() {
        let texts = generate_texts(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &texts, |b, texts| {
            b.iter(|| {
                // 결과를 즉시 드롭하여 메모리 측정
                for text in texts {
                    let tokens = tokenizer.tokenize(black_box(text));
                    drop(black_box(tokens));
                }
            });
        });
    }

    group.finish();
}

/// 실제 사용 시나리오: 소셜 미디어 분석
fn bench_social_media_scenario(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_social_media");

    // 소셜 미디어 스타일 짧은 텍스트 대량
    let posts = vec![
        "좋아요!",
        "감사합니다",
        "오늘 날씨 정말 좋네요",
        "맛있게 먹었어요",
        "추천합니다",
        "최고예요",
        "다음에 또 올게요",
        "정말 재미있었어요",
    ];

    let mut batch = Vec::new();
    for i in 0..1000 {
        batch.push(posts[i % posts.len()].to_string());
    }

    let total_bytes: usize = batch.iter().map(|t| t.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("posts_1000", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for post in &batch {
                let tokens = tokenizer.tokenize(black_box(post));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    group.finish();
}

/// 실제 사용 시나리오: 뉴스 기사 분석
fn bench_news_article_scenario(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_news_articles");

    // 뉴스 기사 스타일 긴 문장
    let articles = vec![
        "대한민국의 수도 서울은 조선시대부터 600년이 넘는 역사를 가진 도시입니다",
        "인공지능 기술의 발전으로 자연어 처리 분야에서도 놀라운 성과가 나타나고 있습니다",
        "한국어는 교착어로 분류되며 조사와 어미가 발달한 언어입니다",
        "형태소 분석은 자연어 처리의 가장 기본적이면서도 중요한 단계입니다",
    ];

    let mut batch = Vec::new();
    for i in 0..100 {
        batch.push(articles[i % articles.len()].to_string());
    }

    let total_bytes: usize = batch.iter().map(|t| t.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    group.bench_function("articles_100", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for article in &batch {
                let tokens = tokenizer.tokenize(black_box(article));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    group.finish();
}

/// 스트리밍 vs 배치 비교
fn bench_streaming_vs_batch(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("batch_streaming_comparison");

    let texts = generate_texts(100);
    let total_bytes: usize = texts.iter().map(|t| t.len()).sum();
    group.throughput(Throughput::Bytes(total_bytes as u64));

    // 배치 방식 (모든 결과 수집)
    group.bench_function("collect_all", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    // 스트리밍 방식 (결과 즉시 처리/드롭)
    group.bench_function("stream_process", |b| {
        b.iter(|| {
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                // 즉시 처리 (여기서는 드롭)
                drop(black_box(tokens));
            }
        });
    });

    group.finish();
}

/// 순차 vs 병렬 처리 비교
fn bench_sequential_vs_parallel(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let batch_tokenizer = BatchTokenizer::new().expect("Failed to create batch tokenizer");
    let mut group = c.benchmark_group("parallel_comparison");

    for size in [100, 500, 1000].iter() {
        let texts = generate_texts(*size);
        let total_bytes: usize = texts.iter().map(|t| t.len()).sum();
        let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

        group.throughput(Throughput::Bytes(total_bytes as u64));

        // 순차 처리
        group.bench_with_input(BenchmarkId::new("sequential", size), &texts, |b, texts| {
            b.iter(|| {
                let mut results = Vec::new();
                for text in texts {
                    let tokens = tokenizer.tokenize(black_box(text));
                    results.push(tokens);
                }
                black_box(results)
            });
        });

        // 병렬 처리 (BatchTokenizer)
        group.bench_with_input(
            BenchmarkId::new("parallel", size),
            &text_refs,
            |b, texts| {
                b.iter(|| {
                    let results = batch_tokenizer.tokenize_batch(black_box(texts));
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

/// 병렬 스케일링 벤치마크 (코어 수에 따른 성능)
fn bench_parallel_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_scaling");
    let texts = generate_texts(1000);
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
    let total_bytes: usize = texts.iter().map(|t| t.len()).sum();

    group.throughput(Throughput::Bytes(total_bytes as u64));
    group.sample_size(10);

    // 다양한 풀 크기로 테스트
    for pool_size in [1, 2, 4, 8].iter() {
        if let Ok(batch) = BatchTokenizer::with_pool_size(*pool_size) {
            group.bench_with_input(
                BenchmarkId::new("pool_size", pool_size),
                &text_refs,
                |b, texts| {
                    b.iter(|| {
                        let results = batch.tokenize_batch(black_box(texts));
                        black_box(results)
                    });
                },
            );
        }
    }

    group.finish();
}

/// 병렬 청크 처리 벤치마크
fn bench_parallel_chunked(c: &mut Criterion) {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");
    let mut group = c.benchmark_group("parallel_chunked");

    // 긴 텍스트 생성
    let long_text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다. ".repeat(100);
    let text_bytes = long_text.len() as u64;

    group.throughput(Throughput::Bytes(text_bytes));

    for chunk_size in [50, 100, 200, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            chunk_size,
            |b, &size| {
                b.iter(|| {
                    let tokens = batch.tokenize_chunked(black_box(&long_text), size);
                    black_box(tokens)
                });
            },
        );
    }

    group.finish();
}

/// 병렬 처리량 측정 (texts/sec)
fn bench_parallel_throughput(c: &mut Criterion) {
    let batch = BatchTokenizer::new().expect("Failed to create batch tokenizer");
    let mut group = c.benchmark_group("parallel_throughput");

    let batch_size = 1000;
    let texts = generate_texts(batch_size);
    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();

    group.throughput(Throughput::Elements(batch_size as u64));
    group.bench_function("texts_per_second_parallel", |b| {
        b.iter(|| {
            let results = batch.tokenize_batch(black_box(&text_refs));
            let total_tokens: usize = results.iter().map(|t| t.len()).sum();
            black_box(total_tokens)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_batches,
    bench_medium_batches,
    bench_large_batches,
    bench_mixed_length_batch,
    bench_throughput_metrics,
    bench_batch_memory_efficiency,
    bench_social_media_scenario,
    bench_news_article_scenario,
    bench_streaming_vs_batch,
    bench_sequential_vs_parallel,
    bench_parallel_scaling,
    bench_parallel_chunked,
    bench_parallel_throughput,
);

criterion_main!(benches);
