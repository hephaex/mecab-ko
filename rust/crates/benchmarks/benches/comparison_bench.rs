//! 비교 벤치마크
//!
//! 측정 항목:
//! - wakati (단어 분리만) vs pos (품사 태깅) vs full tokenization
//! - 사용자 사전 유무 비교
//! - 다양한 출력 형식 비교
//! - 기능별 성능 트레이드오프

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::const_is_empty,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use mecab_ko_core::tokenizer::Tokenizer;

/// wakati vs pos vs full tokenization 비교
fn bench_output_mode_comparison(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_output_modes");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";
    group.throughput(Throughput::Bytes(text.len() as u64));

    // wakati: 단어 분리만 (가장 빠름)
    group.bench_function("wakati", |b| {
        b.iter(|| {
            let morphs = tokenizer.wakati(black_box(text));
            black_box(morphs);
        });
    });

    // pos: 품사 태깅 (중간)
    group.bench_function("pos", |b| {
        b.iter(|| {
            let pos_tags = tokenizer.pos(black_box(text));
            black_box(pos_tags);
        });
    });

    // nouns: 명사 추출 (필터링 포함)
    group.bench_function("nouns", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns);
        });
    });

    // full: 전체 토큰 정보 (가장 느림)
    group.bench_function("full_tokenize", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 다양한 텍스트 길이에서의 출력 모드 비교
fn bench_output_modes_by_length(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_modes_by_length");

    let short = "짧은 텍스트";
    let medium = "중간 길이의 텍스트로 형태소 분석을 수행합니다";
    let long = "이것은 긴 텍스트입니다. \
                한국어 형태소 분석기는 다양한 길이의 텍스트를 처리할 수 있으며, \
                출력 모드에 따라 성능 특성이 달라질 수 있습니다.";

    for (label, text) in [("short", short), ("medium", medium), ("long", long)] {
        // wakati
        group.bench_function(format!("{label}_wakati"), |b| {
            b.iter(|| {
                let morphs = tokenizer.wakati(black_box(text));
                black_box(morphs);
            });
        });

        // full
        group.bench_function(format!("{label}_full"), |b| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            });
        });
    }

    group.finish();
}

/// 사용자 사전 유무 비교 (플레이스홀더)
fn bench_with_without_user_dict(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_user_dict");

    let text = "한국어 형태소 분석기";
    group.throughput(Throughput::Bytes(text.len() as u64));

    // 기본 사전만
    group.bench_function("without_user_dict", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 사용자 사전 포함 (미래 구현)
    // group.bench_function("with_user_dict", |b| { ... });

    group.finish();
}

/// 명사 추출 효율성
fn bench_noun_extraction_efficiency(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_noun_extraction");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";
    group.throughput(Throughput::Bytes(text.len() as u64));

    // 직접 nouns() 호출
    group.bench_function("direct_nouns", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns);
        });
    });

    // tokenize 후 필터링
    group.bench_function("tokenize_then_filter", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            // 명사 필터링 시뮬레이션 (실제 구현은 다를 수 있음)
            let nouns: Vec<_> = tokens.iter().collect();
            black_box(nouns);
        });
    });

    group.finish();
}

/// 다양한 특수화된 처리 비교
fn bench_specialized_processing(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_specialized");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";

    // 기본 토큰화
    group.bench_function("basic_tokenize", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // wakati (문자열만)
    group.bench_function("wakati_strings", |b| {
        b.iter(|| {
            let morphs = tokenizer.wakati(black_box(text));
            black_box(morphs);
        });
    });

    // pos (튜플)
    group.bench_function("pos_tuples", |b| {
        b.iter(|| {
            let pos_tags = tokenizer.pos(black_box(text));
            black_box(pos_tags);
        });
    });

    group.finish();
}

/// 배치 크기별 최적 모드
fn bench_optimal_mode_by_batch(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_batch_modes");

    let texts = vec![
        "첫 번째 문장입니다",
        "두 번째 문장입니다",
        "세 번째 문장입니다",
        "네 번째 문장입니다",
        "다섯 번째 문장입니다",
    ];

    // wakati 배치
    group.bench_function("batch_wakati", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for text in &texts {
                let morphs = tokenizer.wakati(black_box(text));
                results.push(morphs);
            }
            black_box(results);
        });
    });

    // full 배치
    group.bench_function("batch_full", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                results.push(tokens);
            }
            black_box(results);
        });
    });

    group.finish();
}

/// 메모리 사용량 비교
fn bench_memory_by_mode(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_memory_by_mode");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";

    // wakati (작은 메모리)
    group.bench_function("wakati_memory", |b| {
        b.iter(|| {
            let morphs = tokenizer.wakati(black_box(text));
            black_box(morphs);
        });
    });

    // full (큰 메모리)
    group.bench_function("full_memory", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 실제 사용 사례별 최적 모드
fn bench_use_case_optimal_mode(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_use_cases");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";

    // 사용 사례 1: 검색 인덱싱 (명사만 필요)
    group.bench_function("search_indexing", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns);
        });
    });

    // 사용 사례 2: 텍스트 요약 (전체 정보 필요)
    group.bench_function("text_summarization", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 사용 사례 3: 단순 토큰 카운팅
    group.bench_function("token_counting", |b| {
        b.iter(|| {
            let morphs = tokenizer.wakati(black_box(text));
            let count = morphs.len();
            black_box(count);
        });
    });

    group.finish();
}

/// 정확도 vs 속도 트레이드오프
fn bench_accuracy_speed_tradeoff(c: &mut Criterion) {
    let mut group = c.benchmark_group("comparison_accuracy_speed");

    let text = "한국어 형태소 분석기";

    // 빠른 모드 (정확도 낮음, 속도 높음) - 플레이스홀더
    group.bench_function("fast_mode", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 정확 모드 (정확도 높음, 속도 낮음) - 플레이스홀더
    group.bench_function("accurate_mode", |b| {
        let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 다양한 필터 조건 비교
fn bench_filtering_comparison(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("comparison_filtering");

    let text = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";

    // 필터 없음
    group.bench_function("no_filter", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 명사 필터
    group.bench_function("noun_filter", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns);
        });
    });

    // 조사 제거 (미래 구현)
    // group.bench_function("remove_particles", |b| { ... });

    group.finish();
}

criterion_group!(
    benches,
    bench_output_mode_comparison,
    bench_output_modes_by_length,
    bench_with_without_user_dict,
    bench_noun_extraction_efficiency,
    bench_specialized_processing,
    bench_optimal_mode_by_batch,
    bench_memory_by_mode,
    bench_use_case_optimal_mode,
    bench_accuracy_speed_tradeoff,
    bench_filtering_comparison,
);

criterion_main!(benches);
