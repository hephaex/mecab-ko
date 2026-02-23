//! 메모리 사용량 벤치마크
//!
//! 측정 항목:
//! - 토큰화 당 메모리 할당량
//! - 메모리 재사용 효율성
//! - 누수 없이 메모리 해제 확인
//! - 큰 텍스트 처리 시 메모리 확장성

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::unused_self,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    unused_mut,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::tokenizer::Tokenizer;

/// 단일 토큰화 메모리 할당
fn bench_per_tokenization_memory(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_per_tokenization");

    let texts = [
        ("short", "짧은 텍스트"),
        (
            "medium",
            "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다",
        ),
        (
            "long",
            "이것은 매우 긴 텍스트입니다. \
             한국어 형태소 분석기는 다양한 길이의 텍스트를 효율적으로 처리해야 하며, \
             메모리 사용량도 최적화되어야 합니다. \
             이 벤치마크는 텍스트 길이에 따른 메모리 할당 패턴을 측정합니다.",
        ),
    ];

    for (label, text) in &texts {
        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_function(*label, |b| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(*text));
                black_box(tokens);
            });
        });
    }

    group.finish();
}

/// 연속 토큰화 메모리 재사용
fn bench_memory_reuse(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_reuse");

    let text = "한국어 형태소 분석";

    // 단일 토큰화 (할당 + 해제)
    group.bench_function("single_allocation", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 연속 10회 토큰화 (재사용 가능성)
    group.bench_function("sequential_10", |b| {
        b.iter(|| {
            for _ in 0..10 {
                let tokens = tokenizer.tokenize(black_box(text));
                drop(black_box(tokens));
            }
        });
    });

    // 연속 100회 토큰화
    group.bench_function("sequential_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let tokens = tokenizer.tokenize(black_box(text));
                drop(black_box(tokens));
            }
        });
    });

    group.finish();
}

/// 메모리 누적 테스트 (메모리 누수 검사)
fn bench_memory_accumulation(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_accumulation");

    let text = "한국어 형태소 분석기";

    // 결과 즉시 드롭
    group.bench_function("immediate_drop", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let tokens = tokenizer.tokenize(black_box(text));
                drop(black_box(tokens));
            }
        });
    });

    // 결과 누적 후 일괄 드롭
    group.bench_function("batch_drop", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for _ in 0..1000 {
                let tokens = tokenizer.tokenize(black_box(text));
                results.push(tokens);
            }
            drop(black_box(results));
        });
    });

    group.finish();
}

/// 다양한 크기 텍스트 메모리 확장성
fn bench_memory_scalability(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_scalability");

    // 텍스트 크기를 점진적으로 증가
    for &char_count in &[10, 50, 100, 500, 1000, 5000] {
        let text = "가".repeat(char_count);

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(char_count), &text, |b, text| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            });
        });
    }

    group.finish();
}

/// Tokenizer 인스턴스 메모리
fn bench_tokenizer_instance_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_tokenizer_instance");
    group.sample_size(20);

    // 단일 인스턴스
    group.bench_function("single_instance", |b| {
        b.iter(|| {
            let mut tokenizer = Tokenizer::new().expect("Failed");
            black_box(tokenizer);
        });
    });

    // 다중 인스턴스 (동시 3개)
    group.bench_function("three_instances", |b| {
        b.iter(|| {
            let t1 = Tokenizer::new().expect("Failed");
            let t2 = Tokenizer::new().expect("Failed");
            let t3 = Tokenizer::new().expect("Failed");
            black_box((t1, t2, t3));
        });
    });

    group.finish();
}

/// 결과 데이터 크기
fn bench_result_size_overhead(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_result_size");

    let texts = [
        "가",
        "가나다",
        "한국어 형태소",
        "한국어 형태소 분석기는 자연어 처리의 핵심입니다",
    ];

    for (idx, &text) in texts.iter().enumerate() {
        group.bench_function(format!("text_{idx}"), |b| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                let count = tokens.len();
                black_box((tokens, count));
            });
        });
    }

    group.finish();
}

/// 긴 텍스트 스트림 처리
fn bench_long_text_streaming(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_long_text_stream");

    // 매우 긴 텍스트 생성
    let long_text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다. ".repeat(100);

    group.throughput(Throughput::Bytes(long_text.len() as u64));

    // 전체 텍스트 한 번에 처리
    group.bench_function("process_all_at_once", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(&long_text));
            black_box(tokens);
        });
    });

    // 청크로 나누어 처리
    group.bench_function("process_in_chunks", |b| {
        let chunks: Vec<&str> = long_text
            .as_bytes()
            .chunks(long_text.len() / 10)
            .filter_map(|chunk| std::str::from_utf8(chunk).ok())
            .collect();

        b.iter(|| {
            for chunk in &chunks {
                let tokens = tokenizer.tokenize(black_box(chunk));
                drop(black_box(tokens));
            }
        });
    });

    group.finish();
}

/// 메모리 압력 하에서의 성능
fn bench_under_memory_pressure(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_pressure");

    let text = "한국어 형태소 분석기는 자연어 처리 기술입니다";

    // 정상 조건
    group.bench_function("normal", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 많은 임시 할당과 함께
    group.bench_function("with_temp_allocations", |b| {
        b.iter(|| {
            // 임시 메모리 할당
            let _temp1 = vec![0u8; 1024];
            let _temp2 = String::from("temporary allocation");
            let _temp3 = vec!["temp"; 100];

            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 반복 사용 패턴 (웹 서버 시뮬레이션)
fn bench_web_server_pattern(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("memory_web_server_pattern");

    let requests = [
        "첫 번째 요청",
        "두 번째 요청",
        "세 번째 요청",
        "네 번째 요청",
        "다섯 번째 요청",
    ];

    // 요청당 새 메모리 할당
    group.bench_function("per_request_allocation", |b| {
        b.iter(|| {
            for request in &requests {
                let tokens = tokenizer.tokenize(black_box(request));
                // 응답 후 즉시 메모리 해제
                drop(black_box(tokens));
            }
        });
    });

    // 결과 재사용 시도 (벡터 재사용)
    group.bench_function("reuse_result_buffer", |b| {
        b.iter(|| {
            let mut buffer = Vec::new();
            for request in &requests {
                let tokens = tokenizer.tokenize(black_box(request));
                buffer.push(tokens);
                buffer.clear(); // 재사용
            }
            black_box(buffer);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_per_tokenization_memory,
    bench_memory_reuse,
    bench_memory_accumulation,
    bench_memory_scalability,
    bench_tokenizer_instance_memory,
    bench_result_size_overhead,
    bench_long_text_streaming,
    bench_under_memory_pressure,
    bench_web_server_pattern,
);

criterion_main!(benches);
