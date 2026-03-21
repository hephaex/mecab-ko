//! 사전 로딩 모드 벤치마크
//!
//! Eager vs Lazy 로딩 모드 비교:
//! - 초기화 시간
//! - 첫 번째 조회 시간
//! - 연속 조회 시간
//! - 캐시 효과

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    unused_imports,
    unused_mut,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};

/// Eager vs Lazy 로딩 초기화 시간 비교
fn bench_loading_mode_init(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_loading_init");
    group.sample_size(10); // 사전 로딩은 느리므로 샘플 크기 축소

    // Eager 모드 (전체 메모리 로드)
    group.bench_function("eager_load", |b| {
        b.iter(|| {
            let dict = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::speed_optimized(),
            )
            .expect("Failed to load dictionary");
            black_box(dict);
        });
    });

    // Lazy 모드 (메모리 최적화)
    group.bench_function("lazy_load", |b| {
        b.iter(|| {
            let dict = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::default(),
            )
            .expect("Failed to load dictionary");
            black_box(dict);
        });
    });

    // Memory optimized 모드 (mmap + lazy)
    group.bench_function("memory_optimized_load", |b| {
        b.iter(|| {
            let dict = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::memory_optimized(),
            )
            .expect("Failed to load dictionary");
            black_box(dict);
        });
    });

    group.finish();
}

/// 첫 번째 조회 성능 (cold cache)
fn bench_first_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_first_lookup");
    group.sample_size(10);

    // Eager 모드 첫 조회
    group.bench_function("eager_first_lookup", |b| {
        b.iter_batched(
            || {
                SystemDictionary::load_with_options(
                    get_dicdir(),
                    LoadOptions::speed_optimized(),
                )
                .expect("Failed to load dictionary")
            },
            |dict| {
                let results = dict.common_prefix_search(black_box("한국어")).unwrap_or_default();
                black_box(results);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    // Lazy 모드 첫 조회
    group.bench_function("lazy_first_lookup", |b| {
        b.iter_batched(
            || {
                SystemDictionary::load_with_options(
                    get_dicdir(),
                    LoadOptions::default(),
                )
                .expect("Failed to load dictionary")
            },
            |dict| {
                let results = dict.common_prefix_search(black_box("한국어")).unwrap_or_default();
                black_box(results);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    group.finish();
}

/// 연속 조회 성능 (warm cache)
fn bench_warm_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_warm_lookup");

    let words = [
        "한국어",
        "형태소",
        "분석기",
        "자연어",
        "처리",
        "기술",
        "개발",
        "프로그램",
    ];

    // Eager 모드
    {
        let dict = SystemDictionary::load_with_options(
            get_dicdir(),
            LoadOptions::speed_optimized(),
        )
        .expect("Failed to load dictionary");

        // 워밍업
        for word in &words {
            let _ = dict.common_prefix_search(word);
        }

        group.bench_function("eager_warm_lookup", |b| {
            b.iter(|| {
                for word in &words {
                    let results = dict.common_prefix_search(black_box(*word)).unwrap_or_default();
                    black_box(results);
                }
            });
        });
    }

    // Lazy 모드
    {
        let dict = SystemDictionary::load_with_options(
            get_dicdir(),
            LoadOptions::default(),
        )
        .expect("Failed to load dictionary");

        // 워밍업
        for word in &words {
            let _ = dict.common_prefix_search(word);
        }

        group.bench_function("lazy_warm_lookup", |b| {
            b.iter(|| {
                for word in &words {
                    let results = dict.common_prefix_search(black_box(*word)).unwrap_or_default();
                    black_box(results);
                }
            });
        });
    }

    group.finish();
}

/// 캐시 효율성 테스트 (반복 조회)
fn bench_cache_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_cache_efficiency");

    // 동일 단어 반복 조회
    let word = "한국어";

    // Eager 모드
    {
        let dict = SystemDictionary::load_with_options(
            get_dicdir(),
            LoadOptions::speed_optimized(),
        )
        .expect("Failed to load dictionary");

        group.bench_function("eager_repeat_100", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let results = dict.common_prefix_search(black_box(word)).unwrap_or_default();
                    black_box(results);
                }
            });
        });
    }

    // Lazy 모드 (캐시 크기 기본값)
    {
        let dict = SystemDictionary::load_with_options(
            get_dicdir(),
            LoadOptions::default(),
        )
        .expect("Failed to load dictionary");

        group.bench_function("lazy_repeat_100", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let results = dict.common_prefix_search(black_box(word)).unwrap_or_default();
                    black_box(results);
                }
            });
        });
    }

    // Lazy 모드 (작은 캐시)
    {
        let dict = SystemDictionary::load_with_options(
            get_dicdir(),
            LoadOptions {
                use_mmap_matrix: false,
                use_lazy_entries: true,
                lazy_cache_size: Some(100), // 작은 캐시
            },
        )
        .expect("Failed to load dictionary");

        group.bench_function("lazy_small_cache_repeat_100", |b| {
            b.iter(|| {
                for _ in 0..100 {
                    let results = dict.common_prefix_search(black_box(word)).unwrap_or_default();
                    black_box(results);
                }
            });
        });
    }

    group.finish();
}

/// 다중 인스턴스 메모리 공유 테스트
fn bench_multiple_instances(c: &mut Criterion) {
    let mut group = c.benchmark_group("dict_multiple_instances");
    group.sample_size(10);

    // Eager 모드 3개 인스턴스
    group.bench_function("eager_3_instances", |b| {
        b.iter(|| {
            let d1 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::speed_optimized(),
            )
            .expect("Failed");
            let d2 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::speed_optimized(),
            )
            .expect("Failed");
            let d3 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::speed_optimized(),
            )
            .expect("Failed");
            black_box((d1, d2, d3));
        });
    });

    // Lazy 모드 3개 인스턴스
    group.bench_function("lazy_3_instances", |b| {
        b.iter(|| {
            let d1 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::default(),
            )
            .expect("Failed");
            let d2 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::default(),
            )
            .expect("Failed");
            let d3 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::default(),
            )
            .expect("Failed");
            black_box((d1, d2, d3));
        });
    });

    // Memory optimized 모드 3개 인스턴스 (mmap 공유)
    group.bench_function("mmap_3_instances", |b| {
        b.iter(|| {
            let d1 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::memory_optimized(),
            )
            .expect("Failed");
            let d2 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::memory_optimized(),
            )
            .expect("Failed");
            let d3 = SystemDictionary::load_with_options(
                get_dicdir(),
                LoadOptions::memory_optimized(),
            )
            .expect("Failed");
            black_box((d1, d2, d3));
        });
    });

    group.finish();
}

/// 사전 디렉토리 경로 가져오기
fn get_dicdir() -> std::path::PathBuf {
    use mecab_ko_dict::DictionaryLoader;

    DictionaryLoader::find_dicdir()
        .expect("Dictionary not found. Set MECAB_DICDIR environment variable.")
}

criterion_group!(
    benches,
    bench_loading_mode_init,
    bench_first_lookup,
    bench_warm_lookup,
    bench_cache_efficiency,
    bench_multiple_instances,
);

criterion_main!(benches);
