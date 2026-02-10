//! Trie 검색 성능 벤치마크
//!
//! 측정 항목:
//! - `exact_match`: 정확한 키 검색
//! - `common_prefix_search`: 공통 접두사 검색
//! - 대용량 사전에서의 성능
//! - 한글 특화 패턴 검색

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::doc_markdown,
    missing_docs
)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_dict::trie::{Trie, TrieBuilder};

/// 테스트용 사전 엔트리 생성
fn create_small_dictionary() -> Vec<(&'static str, u32)> {
    vec![
        ("가", 0),
        ("가다", 1),
        ("가방", 2),
        ("가방에", 3),
        ("나", 4),
        ("나다", 5),
        ("다", 6),
        ("다가", 7),
        ("다가가다", 8),
        ("아버지", 9),
        ("아버지가", 10),
        ("어머니", 11),
        ("어머니는", 12),
        ("학교", 13),
        ("학교에", 14),
        ("학생", 15),
        ("선생님", 16),
    ]
}

/// 중형 사전 생성 (1000개 엔트리)
fn create_medium_dictionary() -> Vec<(String, u32)> {
    let mut entries = Vec::new();

    // 명사 패턴
    let nouns = [
        "학교",
        "학생",
        "선생님",
        "교실",
        "책상",
        "의자",
        "칠판",
        "공책",
        "연필",
        "지우개",
        "가방",
        "필통",
        "운동장",
        "도서관",
        "식당",
        "복도",
        "계단",
        "화장실",
        "교무실",
        "과학실",
    ];

    // 동사 어간
    let verbs = ["가", "오", "먹", "자", "하", "보", "듣", "말하", "읽", "쓰"];

    // 명사 조합
    for (i, noun) in nouns.iter().enumerate() {
        entries.push(((*noun).to_string(), i as u32));
        entries.push((format!("{noun}에"), (i + 100) as u32));
        entries.push((format!("{noun}의"), (i + 200) as u32));
        entries.push((format!("{noun}를"), (i + 300) as u32));
        entries.push((format!("{noun}은"), (i + 400) as u32));
    }

    // 동사 활용
    for (i, verb) in verbs.iter().enumerate() {
        let base = 500 + i * 10;
        entries.push((format!("{verb}다"), base as u32));
        entries.push((format!("{verb}고"), (base + 1) as u32));
        entries.push((format!("{verb}면"), (base + 2) as u32));
        entries.push((format!("{verb}니"), (base + 3) as u32));
        entries.push((format!("{verb}어"), (base + 4) as u32));
        entries.push((format!("{verb}아"), (base + 5) as u32));
    }

    // 복합어
    for i in 0..100 {
        entries.push((format!("복합어{i}"), (1000 + i) as u32));
    }

    entries
}

/// 대형 사전 생성 (10000개 엔트리)
fn create_large_dictionary() -> Vec<(String, u32)> {
    let mut entries = create_medium_dictionary();

    // 추가 엔트리 생성
    for i in 0..9000 {
        entries.push((format!("단어{i:04}"), (10000 + i) as u32));
    }

    entries
}

/// 소형 사전 - `exact_match` 벤치마크
fn bench_exact_match_small(c: &mut Criterion) {
    let mut entries = create_small_dictionary();
    let bytes = TrieBuilder::build_unsorted(&mut entries).expect("Failed to build trie");
    let trie = Trie::new(&bytes);

    let mut group = c.benchmark_group("trie_exact_match_small");

    group.bench_function("hit", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("아버지가")));
        });
    });

    group.bench_function("miss", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("존재하지않는단어")));
        });
    });

    group.bench_function("batch_10", |b| {
        let queries = [
            "가",
            "나다",
            "학교",
            "선생님",
            "아버지",
            "없음1",
            "없음2",
            "가방에",
            "학생",
            "없음3",
        ];
        b.iter(|| {
            for query in &queries {
                black_box(trie.exact_match(black_box(*query)));
            }
        });
    });

    group.finish();
}

/// 중형 사전 - `exact_match` 벤치마크
fn bench_exact_match_medium(c: &mut Criterion) {
    let mut entries = create_medium_dictionary();
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let bytes = TrieBuilder::build(
        &entries
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect::<Vec<_>>(),
    )
    .expect("Failed to build trie");
    let trie = Trie::new(&bytes);

    let mut group = c.benchmark_group("trie_exact_match_medium");
    group.throughput(Throughput::Elements(1));

    group.bench_function("hit_short", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("가다")));
        });
    });

    group.bench_function("hit_long", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("선생님의")));
        });
    });

    group.bench_function("miss", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("존재하지않는긴단어입니다")));
        });
    });

    group.finish();
}

/// 대형 사전 - `exact_match` 벤치마크
fn bench_exact_match_large(c: &mut Criterion) {
    let mut entries = create_large_dictionary();
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let bytes = TrieBuilder::build(
        &entries
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect::<Vec<_>>(),
    )
    .expect("Failed to build trie");
    let trie = Trie::new(&bytes);

    let mut group = c.benchmark_group("trie_exact_match_large");
    group.throughput(Throughput::Elements(1));

    group.bench_function("hit", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("학교에")));
        });
    });

    group.bench_function("miss", |b| {
        b.iter(|| {
            black_box(trie.exact_match(black_box("미등록어")));
        });
    });

    group.finish();
}

/// `common_prefix_search` 벤치마크
fn bench_common_prefix_search(c: &mut Criterion) {
    let mut entries = create_small_dictionary();
    let bytes = TrieBuilder::build_unsorted(&mut entries).expect("Failed to build trie");
    let trie = Trie::new(&bytes);

    let mut group = c.benchmark_group("trie_common_prefix_search");

    // "가방에서" → ["가", "가방", "가방에"] 매칭
    group.bench_function("multi_match", |b| {
        b.iter(|| {
            let results: Vec<_> = trie.common_prefix_search(black_box("가방에서")).collect();
            black_box(results);
        });
    });

    // "아버지가방에" → ["아버지", "아버지가"] 등
    group.bench_function("long_text", |b| {
        b.iter(|| {
            let results: Vec<_> = trie
                .common_prefix_search(black_box("아버지가방에"))
                .collect();
            black_box(results);
        });
    });

    // 매칭 없는 경우
    group.bench_function("no_match", |b| {
        b.iter(|| {
            let results: Vec<_> = trie.common_prefix_search(black_box("없는단어")).collect();
            black_box(results);
        });
    });

    group.finish();
}

/// 실제 형태소 분석 시나리오
fn bench_morpheme_analysis_scenario(c: &mut Criterion) {
    let mut entries = create_medium_dictionary();
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

    let bytes = TrieBuilder::build(
        &entries
            .iter()
            .map(|(k, v)| (k.as_str(), *v))
            .collect::<Vec<_>>(),
    )
    .expect("Failed to build trie");
    let trie = Trie::new(&bytes);

    let mut group = c.benchmark_group("trie_morpheme_scenario");

    // 문장 전체 처리 (각 위치에서 prefix search)
    group.bench_function("sentence_analysis", |b| {
        let text = "학생이교실에가다";
        b.iter(|| {
            let mut total_matches = 0;
            for i in 0..text.chars().count() {
                // 각 문자 위치에서 검색
                let byte_pos = text
                    .char_indices()
                    .nth(i)
                    .map_or(text.len(), |(pos, _)| pos);
                let results = trie.common_prefix_search_at(black_box(text), byte_pos);
                total_matches += results.len();
            }
            black_box(total_matches);
        });
    });

    group.finish();
}

/// Trie 빌드 성능
fn bench_trie_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("trie_build");

    // 소형 사전 빌드
    group.bench_function("small_sorted", |b| {
        let mut entries = create_small_dictionary();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        b.iter(|| {
            let bytes = TrieBuilder::build(black_box(&entries)).expect("Failed to build");
            black_box(bytes);
        });
    });

    group.bench_function("small_unsorted", |b| {
        let entries = create_small_dictionary();

        b.iter(|| {
            let mut e = entries.clone();
            let bytes = TrieBuilder::build_unsorted(black_box(&mut e)).expect("Failed to build");
            black_box(bytes);
        });
    });

    // 중형 사전 빌드
    group.bench_function("medium", |b| {
        let mut entries = create_medium_dictionary();
        entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        let refs: Vec<_> = entries.iter().map(|(k, v)| (k.as_str(), *v)).collect();

        b.iter(|| {
            let bytes = TrieBuilder::build(black_box(&refs)).expect("Failed to build");
            black_box(bytes);
        });
    });

    group.finish();
}

/// 메모리 효율성 테스트
fn bench_trie_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("trie_memory");

    for size in &[100, 1000, 5000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut entries: Vec<(String, u32)> = (0..size)
                .map(|i| (format!("단어{i:05}"), i as u32))
                .collect();
            entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));

            let refs: Vec<_> = entries.iter().map(|(k, v)| (k.as_str(), *v)).collect();

            b.iter(|| {
                let bytes = TrieBuilder::build(black_box(&refs)).expect("Failed to build");

                // 압축률 측정을 위한 원본 크기 계산
                let original_size: usize = entries
                    .iter()
                    .map(|(k, _)| k.len() + std::mem::size_of::<u32>())
                    .sum();

                black_box((bytes.len(), original_size));
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_exact_match_small,
    bench_exact_match_medium,
    bench_exact_match_large,
    bench_common_prefix_search,
    bench_morpheme_analysis_scenario,
    bench_trie_build,
    bench_trie_memory_efficiency,
);

criterion_main!(benches);
