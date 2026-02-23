//! Analyzer 벤치마크

#![allow(missing_docs, clippy::unwrap_used)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};

fn analyzer_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("analyzer");

    // 짧은 텍스트
    let short_text = "한국어 형태소 분석기";

    // 중간 길이 텍스트
    let medium_text = "한국어 형태소 분석기는 자연어 처리의 기본 도구입니다. \
                       이를 통해 텍스트를 의미 있는 단위로 분해할 수 있습니다.";

    // 긴 텍스트
    let long_text = "한국어 형태소 분석기는 자연어 처리의 기본 도구입니다. \
                     이를 통해 텍스트를 의미 있는 단위로 분해할 수 있습니다. \
                     Elasticsearch와 통합하여 강력한 검색 기능을 제공합니다. \
                     복합명사 분해, 사용자 사전, 품사 필터링 등 다양한 기능을 지원합니다. \
                     Lucene Nori와 호환되는 인터페이스를 제공하여 쉽게 마이그레이션할 수 있습니다.";

    // 짧은 텍스트 벤치마크
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
    group.throughput(Throughput::Bytes(short_text.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("short_text", "none"),
        &short_text,
        |b, text| {
            b.iter(|| analyzer.analyze(black_box(text)));
        },
    );

    // 중간 텍스트 벤치마크
    group.throughput(Throughput::Bytes(medium_text.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("medium_text", "none"),
        &medium_text,
        |b, text| {
            b.iter(|| analyzer.analyze(black_box(text)));
        },
    );

    // 긴 텍스트 벤치마크
    group.throughput(Throughput::Bytes(long_text.len() as u64));
    group.bench_with_input(
        BenchmarkId::new("long_text", "none"),
        &long_text,
        |b, text| {
            b.iter(|| analyzer.analyze(black_box(text)));
        },
    );

    group.finish();
}

fn decompound_mode_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompound_mode");

    let text = "형태소분석기를 사용하여 자연어처리를 수행합니다.";

    // None 모드
    let analyzer_none = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
    group.bench_with_input(BenchmarkId::new("mode", "none"), &text, |b, text| {
        b.iter(|| analyzer_none.analyze(black_box(text)));
    });

    // Discard 모드
    let analyzer_discard = NoriAnalyzer::default_with_decompound(DecompoundMode::Discard).unwrap();
    group.bench_with_input(BenchmarkId::new("mode", "discard"), &text, |b, text| {
        b.iter(|| analyzer_discard.analyze(black_box(text)));
    });

    // Mixed 모드
    let analyzer_mixed = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed).unwrap();
    group.bench_with_input(BenchmarkId::new("mode", "mixed"), &text, |b, text| {
        b.iter(|| analyzer_mixed.analyze(black_box(text)));
    });

    group.finish();
}

fn filter_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter");

    let text = "한국어 형태소 분석기를 사용하여 자연어 처리를 수행합니다.";

    // 필터 없음
    let config_no_filter = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec![]);
    let analyzer_no_filter = NoriAnalyzer::new(config_no_filter).unwrap();

    group.bench_with_input(
        BenchmarkId::new("stoptags", "no_filter"),
        &text,
        |b, text| {
            b.iter(|| analyzer_no_filter.analyze(black_box(text)));
        },
    );

    // 기본 필터 (조사, 어미)
    let analyzer_with_filter = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();

    group.bench_with_input(
        BenchmarkId::new("stoptags", "with_filter"),
        &text,
        |b, text| {
            b.iter(|| analyzer_with_filter.analyze(black_box(text)));
        },
    );

    group.finish();
}

fn analyzer_creation_benchmark(c: &mut Criterion) {
    c.bench_function("analyzer_creation", |b| {
        b.iter(|| {
            let config = AnalyzerConfig::new().with_decompound_mode(DecompoundMode::None);
            NoriAnalyzer::new(black_box(config))
        });
    });
}

fn concurrent_analysis_benchmark(c: &mut Criterion) {
    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::None).unwrap();
    let text = "한국어 형태소 분석기";

    c.bench_function("concurrent_analysis", |b| {
        b.iter(|| {
            // 동일 analyzer로 여러 번 분석
            for _ in 0..10 {
                let _ = analyzer.analyze(black_box(text));
            }
        });
    });
}

fn throughput_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    let analyzer = NoriAnalyzer::default_with_decompound(DecompoundMode::Mixed).unwrap();

    // 다양한 텍스트 크기로 처리량 측정
    for size in [100, 500, 1000, 5000] {
        let text = "한국어 형태소 분석기를 사용하여 자연어 처리를 수행합니다. ".repeat(size / 50);

        group.throughput(Throughput::Bytes(text.len() as u64));
        group.bench_with_input(BenchmarkId::new("chars", size), &text, |b, text| {
            b.iter(|| analyzer.analyze(black_box(text)));
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    analyzer_benchmark,
    decompound_mode_benchmark,
    filter_benchmark,
    analyzer_creation_benchmark,
    concurrent_analysis_benchmark,
    throughput_benchmark
);
criterion_main!(benches);
