//! # 메모리 사용량 벤치마크
//!
//! 메모리 할당, 재사용, 복사 비용 등을 측정하는 벤치마크

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::tokenizer::Token;
use mecab_ko_core::Tokenizer;

/// 토큰 복사 비용 벤치마크
fn bench_token_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_clone");

    let token = Token::new("안녕하세요".to_string(), "NNG".to_string(), 0, 5, 0, 15);

    group.bench_function("single_clone", |b| {
        b.iter(|| {
            let cloned = token.clone();
            black_box(cloned)
        })
    });

    // 여러 토큰 복사
    let tokens: Vec<Token> = (0..100)
        .map(|i| {
            Token::new(
                format!("토큰{i}"),
                "NNG".to_string(),
                i,
                i + 1,
                i * 10,
                (i + 1) * 10,
            )
        })
        .collect();

    group.bench_function("vec_clone_100", |b| {
        b.iter(|| {
            let cloned = tokens.clone();
            black_box(cloned)
        })
    });

    group.finish();
}

/// 문자열 할당 비용 벤치마크
fn bench_string_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_allocation");

    group.bench_function("short_string", |b| {
        b.iter(|| {
            let s = "안녕".to_string();
            black_box(s)
        })
    });

    group.bench_function("medium_string", |b| {
        b.iter(|| {
            let s = "안녕하세요 반갑습니다".to_string();
            black_box(s)
        })
    });

    group.bench_function("long_string", |b| {
        b.iter(|| {
            let s =
                "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다".to_string();
            black_box(s)
        })
    });

    group.finish();
}

/// Vec 할당 패턴 벤치마크
fn bench_vec_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("vec_allocation");

    // 용량 없이 생성
    group.bench_function("vec_new", |b| {
        b.iter(|| {
            let mut vec: Vec<Token> = Vec::new();
            for i in 0..10 {
                vec.push(Token::new(
                    format!("토큰{i}"),
                    "NNG".to_string(),
                    i,
                    i + 1,
                    i * 10,
                    (i + 1) * 10,
                ));
            }
            black_box(vec)
        })
    });

    // 용량 사전 할당
    group.bench_function("vec_with_capacity", |b| {
        b.iter(|| {
            let mut vec: Vec<Token> = Vec::with_capacity(10);
            for i in 0..10 {
                vec.push(Token::new(
                    format!("토큰{i}"),
                    "NNG".to_string(),
                    i,
                    i + 1,
                    i * 10,
                    (i + 1) * 10,
                ));
            }
            black_box(vec)
        })
    });

    group.finish();
}

/// 토크나이저 메모리 재사용 벤치마크
fn bench_tokenizer_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_reuse");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    let texts = vec![
        "안녕하세요",
        "오늘 날씨가 좋습니다",
        "반갑습니다",
        "감사합니다",
        "좋은 하루 보내세요",
    ];

    // 토크나이저 재사용
    group.bench_function("reuse_tokenizer", |b| {
        b.iter(|| {
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            }
        })
    });

    // 매번 새로 생성
    group.bench_function("create_new_each_time", |b| {
        b.iter(|| {
            for text in &texts {
                if let Ok(mut new_tokenizer) = Tokenizer::new() {
                    let tokens = new_tokenizer.tokenize(black_box(text));
                    black_box(tokens);
                }
            }
        })
    });

    group.finish();
}

/// 대량 토큰 생성 시 메모리 사용 패턴
fn bench_bulk_token_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_token_creation");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    // 긴 텍스트 (많은 토큰 생성)
    let long_text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다. \
                    특히 한국어 형태소 분석은 교착어의 특성상 매우 복잡한 과정을 거치게 됩니다. \
                    MeCab은 일본어 형태소 분석기로 시작되었지만, 한국어에도 적용되어 \
                    높은 성능과 정확도를 보여주고 있습니다."
        .repeat(5);

    let byte_size = long_text.len();
    group.throughput(Throughput::Bytes(byte_size as u64));

    group.bench_function("large_text", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(&long_text));
            black_box(tokens)
        })
    });

    group.finish();
}

/// 문자열 intern 효과 측정
fn bench_string_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interning");

    // 동일한 품사 태그가 반복되는 경우
    let pos_tags = vec!["NNG", "JKS", "VV", "EC", "NNG", "JKS", "VV", "EC"];

    // 매번 새로 할당
    group.bench_function("always_allocate", |b| {
        b.iter(|| {
            let allocated: Vec<String> = pos_tags.iter().map(|s| s.to_string()).collect();
            black_box(allocated)
        })
    });

    // 문자열 슬라이스로 참조
    group.bench_function("use_slices", |b| {
        b.iter(|| {
            let slices: Vec<&str> = pos_tags.iter().copied().collect();
            black_box(slices)
        })
    });

    group.finish();
}

/// 토큰 벡터 크기별 메모리 패턴
fn bench_token_vec_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_vec_sizes");

    for size in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let tokens: Vec<Token> = (0..size)
                    .map(|i| {
                        Token::new(
                            format!("토큰{i}"),
                            "NNG".to_string(),
                            i,
                            i + 1,
                            i * 10,
                            (i + 1) * 10,
                        )
                    })
                    .collect();
                black_box(tokens)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_token_clone,
    bench_string_allocation,
    bench_vec_allocation,
    bench_tokenizer_reuse,
    bench_bulk_token_creation,
    bench_string_interning,
    bench_token_vec_sizes,
);

criterion_main!(benches);
