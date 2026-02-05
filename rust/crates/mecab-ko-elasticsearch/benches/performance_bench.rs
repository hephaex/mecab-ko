//! 종합 성능 벤치마크
//!
//! 실제 사용 시나리오를 시뮬레이션하여 성능 측정

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};

/// 실제 문서 샘플 (뉴스 기사 스타일)
const REALISTIC_DOCUMENT: &str = "
서울특별시는 대한민국의 수도이자 최대 도시입니다. \
인구는 약 천만 명에 달하며, 경제, 정치, 문화의 중심지로 기능하고 있습니다. \
한강을 중심으로 강북과 강남으로 나뉘며, 25개 자치구로 구성되어 있습니다. \
조선시대부터 수도로서의 역할을 해왔으며, 600년이 넘는 역사를 자랑합니다. \
현대에는 IT 산업과 금융 산업이 발달하여 글로벌 경쟁력을 갖춘 도시로 성장했습니다. \
주요 관광지로는 경복궁, 남산타워, 명동, 강남역 등이 있으며, \
매년 수백만 명의 외국인 관광객이 방문하고 있습니다.
";

/// 짧은 검색 쿼리 모음
const SHORT_QUERIES: &[&str] = &[
    "한국어",
    "형태소 분석",
    "자연어 처리",
    "검색 엔진",
    "머신러닝",
    "딥러닝 모델",
    "빅데이터 분석",
    "클라우드 컴퓨팅",
];

/// 중간 길이 문장 모음
const MEDIUM_SENTENCES: &[&str] = &[
    "한국어 형태소 분석기는 자연어 처리의 핵심 도구입니다.",
    "Elasticsearch와 통합하여 강력한 검색 기능을 제공합니다.",
    "복합명사 분해와 품사 태그 필터링을 지원합니다.",
    "사용자 사전을 통해 도메인 특화 용어를 처리할 수 있습니다.",
    "실시간 분석과 배치 처리 모두 지원합니다.",
];

/// 단일 문서 분석 (캐시 미사용)
fn bench_single_document_no_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_document");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::without_cache(config).unwrap();

    group.throughput(Throughput::Bytes(REALISTIC_DOCUMENT.len() as u64));
    group.bench_function("no_cache", |b| {
        b.iter(|| analyzer.analyze(black_box(REALISTIC_DOCUMENT)));
    });

    group.finish();
}

/// 단일 문서 분석 (캐시 사용)
fn bench_single_document_with_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_document_cache");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::with_cache_size(config, 1024).unwrap();

    group.throughput(Throughput::Bytes(REALISTIC_DOCUMENT.len() as u64));

    // 첫 번째 실행 (캐시 미스)
    group.bench_function("first_run", |b| {
        b.iter(|| {
            analyzer.clear_cache();
            analyzer.analyze(black_box(REALISTIC_DOCUMENT))
        });
    });

    // 두 번째 실행 (캐시 히트)
    group.bench_function("cached_run", |b| {
        // Pre-warm cache
        let _ = analyzer.analyze(REALISTIC_DOCUMENT);

        b.iter(|| analyzer.analyze(black_box(REALISTIC_DOCUMENT)));
    });

    group.finish();
}

/// 짧은 쿼리 분석 (검색 시나리오)
fn bench_short_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("short_queries");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::with_cache_size(config, 100).unwrap();

    // 평균 쿼리 길이
    let avg_len: usize = SHORT_QUERIES.iter().map(|q| q.len()).sum::<usize>() / SHORT_QUERIES.len();

    group.throughput(Throughput::Bytes(avg_len as u64));

    group.bench_function("query_processing", |b| {
        let mut idx = 0;
        b.iter(|| {
            let query = SHORT_QUERIES[idx % SHORT_QUERIES.len()];
            idx += 1;
            analyzer.analyze(black_box(query))
        });
    });

    group.finish();
}

/// 배치 처리 성능
#[cfg(feature = "batch")]
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::without_cache(config).unwrap();

    for size in [10, 50, 100] {
        let total_len: usize = MEDIUM_SENTENCES
            .iter()
            .take(size.min(MEDIUM_SENTENCES.len()))
            .map(|s| s.len())
            .sum();

        group.throughput(Throughput::Bytes(total_len as u64));

        // 순차 처리
        group.bench_with_input(BenchmarkId::new("sequential", size), &size, |b, &size| {
            b.iter(|| {
                let mut results = Vec::new();
                for i in 0..size {
                    let text = MEDIUM_SENTENCES[i % MEDIUM_SENTENCES.len()];
                    results.push(analyzer.analyze(black_box(text)));
                }
                results
            });
        });

        // 병렬 배치 처리
        group.bench_with_input(BenchmarkId::new("parallel", size), &size, |b, &size| {
            b.iter(|| {
                let texts: Vec<_> = (0..size)
                    .map(|i| MEDIUM_SENTENCES[i % MEDIUM_SENTENCES.len()])
                    .collect();
                analyzer.analyze_batch(black_box(&texts))
            });
        });
    }

    group.finish();
}

/// 캐시 히트율 측정
fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_efficiency");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    for cache_size in [0, 10, 100, 1000] {
        let analyzer = NoriAnalyzer::with_cache_size(config.clone(), cache_size).unwrap();

        group.bench_with_input(
            BenchmarkId::new("repeated_queries", cache_size),
            &cache_size,
            |b, _| {
                b.iter(|| {
                    // 동일한 쿼리 반복 (캐시 효과 측정)
                    for _ in 0..10 {
                        for query in SHORT_QUERIES {
                            let _ = analyzer.analyze(black_box(query));
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// 복합명사 분해 모드 비교
fn bench_decompound_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("decompound_modes");

    let text = "형태소분석기를 사용하여 자연어처리를 수행합니다. 복합명사분해 기능이 중요합니다.";

    for mode in [
        DecompoundMode::None,
        DecompoundMode::Discard,
        DecompoundMode::Mixed,
    ] {
        let config = AnalyzerConfig::new()
            .with_decompound_mode(mode)
            .with_stoptags(vec![]);

        let analyzer = NoriAnalyzer::without_cache(config).unwrap();

        group.bench_with_input(BenchmarkId::new("mode", mode.as_str()), &mode, |b, _| {
            b.iter(|| analyzer.analyze(black_box(text)));
        });
    }

    group.finish();
}

/// 필터 체인 성능
fn bench_filter_chain(c: &mut Criterion) {
    let mut group = c.benchmark_group("filter_chain");

    let text = "한국어 형태소 분석기를 사용하여 자연어 처리를 수행합니다.";

    // 필터 없음
    let config_no_filter = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec![]);
    let analyzer_no_filter = NoriAnalyzer::without_cache(config_no_filter).unwrap();

    group.bench_function("no_filter", |b| {
        b.iter(|| analyzer_no_filter.analyze(black_box(text)));
    });

    // 기본 필터 (J, E)
    let config_basic = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);
    let analyzer_basic = NoriAnalyzer::without_cache(config_basic).unwrap();

    group.bench_function("basic_filter", |b| {
        b.iter(|| analyzer_basic.analyze(black_box(text)));
    });

    // 확장 필터 (J, E, SF, SP, SS)
    let config_extended = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::None)
        .with_stoptags(vec![
            "J".to_string(),
            "E".to_string(),
            "SF".to_string(),
            "SP".to_string(),
            "SS".to_string(),
        ]);
    let analyzer_extended = NoriAnalyzer::without_cache(config_extended).unwrap();

    group.bench_function("extended_filter", |b| {
        b.iter(|| analyzer_extended.analyze(black_box(text)));
    });

    group.finish();
}

/// 메모리 사용량 측정 (간접적)
fn bench_memory_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_pressure");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    let analyzer = NoriAnalyzer::without_cache(config).unwrap();

    // 연속 처리 (메모리 재사용 측정)
    group.bench_function("continuous_processing", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = analyzer.analyze(black_box("한국어 형태소 분석"));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_document_no_cache,
    bench_single_document_with_cache,
    bench_short_queries,
    bench_cache_hit_rate,
    bench_decompound_modes,
    bench_filter_chain,
    bench_memory_pressure,
);

#[cfg(feature = "batch")]
criterion_group!(batch_benches, bench_batch_processing);

#[cfg(feature = "batch")]
criterion_main!(benches, batch_benches);

#[cfg(not(feature = "batch"))]
criterion_main!(benches);
