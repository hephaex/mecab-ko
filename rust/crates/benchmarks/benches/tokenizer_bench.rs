//! 전체 토크나이저 성능 벤치마크
//!
//! 측정 항목:
//! - End-to-end 토크나이저 처리량
//! - 다양한 텍스트 길이에서의 성능
//! - 메모리 사용량
//! - Throughput (characters/sec, tokens/sec)
//! - 실제 문장 패턴 성능

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::tokenizer::Tokenizer;

/// 짧은 문장들 (소셜 미디어 스타일)
const SHORT_SENTENCES: &[&str] = &[
    "안녕하세요",
    "오늘 날씨 좋네요",
    "감사합니다",
    "잘 부탁드립니다",
    "좋은 하루 되세요",
];

/// 중간 길이 문장들 (일반 대화)
const MEDIUM_SENTENCES: &[&str] = &[
    "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다",
    "아버지가 방에 들어가신다는 문장을 분석해보겠습니다",
    "오늘은 날씨가 맑고 화창하여 산책하기 좋은 날입니다",
    "형태소 분석 결과는 품사 태깅과 함께 제공됩니다",
    "자연어 처리는 인공지능의 중요한 응용 분야 중 하나입니다",
];

/// 긴 문장들 (뉴스 기사 스타일)
const LONG_SENTENCES: &[&str] = &[
    "대한민국의 수도인 서울은 조선시대부터 600년이 넘는 역사를 가진 도시로서 \
     현대적인 빌딩과 전통 한옥이 조화를 이루며 독특한 도시 경관을 형성하고 있습니다",
    "인공지능 기술의 발전으로 자연어 처리 분야에서도 놀라운 성과들이 나타나고 있으며 \
     특히 대규모 언어 모델의 등장은 기계 번역과 텍스트 생성 분야에 혁신적인 변화를 가져왔습니다",
    "한국어는 교착어로 분류되며 조사와 어미가 발달한 언어로서 형태소 분석의 중요성이 \
     다른 언어들에 비해 더욱 강조되며 이에 따라 정확한 형태소 분석기의 개발이 필수적입니다",
];

/// 기술 문서 (전문 용어 포함)
const TECHNICAL_TEXTS: &[&str] = &[
    "Rust 프로그래밍 언어는 메모리 안전성을 보장하면서도 고성능을 제공합니다",
    "Double-Array Trie는 효율적인 문자열 검색을 위한 자료구조입니다",
    "Viterbi 알고리즘은 HMM에서 최적 경로를 찾는 동적 프로그래밍 기법입니다",
];

/// 혼합 텍스트 (한글, 영문, 숫자, 기호)
const MIXED_TEXTS: &[&str] = &[
    "Apple의 iPhone 15 Pro는 A17 칩을 탑재했습니다",
    "2024년 1월 1일부터 새로운 정책이 시행됩니다",
    "AI(Artificial Intelligence)는 인공지능을 의미합니다",
    "서울시 강남구 역삼동 123-45번지",
];

/// 기본 토크나이저 처리 성능
fn bench_tokenizer_basic(c: &mut Criterion) {
    // Note: 현재 Tokenizer는 스텁 구현이므로 실제 성능 측정은 구현 완료 후 가능
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_basic");

    // 단일 짧은 문장
    let text = SHORT_SENTENCES[0];
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_function("short_single", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 중간 길이 문장
    let text = MEDIUM_SENTENCES[0];
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_function("medium_single", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 긴 문장
    let text = LONG_SENTENCES[0];
    group.throughput(Throughput::Bytes(text.len() as u64));
    group.bench_function("long_single", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 배치 처리 성능
fn bench_tokenizer_batch(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_batch");

    // 짧은 문장 배치
    group.bench_function("short_batch", |b| {
        let total_bytes: usize = SHORT_SENTENCES.iter().map(|s| s.len()).sum();

        b.iter(|| {
            for &text in SHORT_SENTENCES {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            }
        });
    });

    // 중간 길이 문장 배치
    group.bench_function("medium_batch", |b| {
        let total_bytes: usize = MEDIUM_SENTENCES.iter().map(|s| s.len()).sum();

        b.iter(|| {
            for &text in MEDIUM_SENTENCES {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            }
        });
    });

    group.finish();
}

/// 텍스트 길이별 확장성
fn bench_tokenizer_scalability(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_scalability");

    for &length in &[10, 50, 100, 500, 1000] {
        let text = "한국어 ".repeat(length / 4); // ~4 bytes per "한국어 "

        group.bench_with_input(BenchmarkId::new("chars", length), &text, |b, text| {
            group.throughput(Throughput::Bytes(text.len() as u64));

            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            });
        });
    }

    group.finish();
}

/// wakati (분리만) 성능
fn bench_tokenizer_wakati(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_wakati");

    group.bench_function("medium", |b| {
        let text = MEDIUM_SENTENCES[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let morphs = tokenizer.wakati(black_box(text));
            black_box(morphs);
        });
    });

    group.finish();
}

/// pos (품사 태깅) 성능
fn bench_tokenizer_pos(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_pos");

    group.bench_function("medium", |b| {
        let text = MEDIUM_SENTENCES[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let pos_tags = tokenizer.pos(black_box(text));
            black_box(pos_tags);
        });
    });

    group.finish();
}

/// nouns (명사 추출) 성능
fn bench_tokenizer_nouns(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_nouns");

    group.bench_function("medium", |b| {
        let text = MEDIUM_SENTENCES[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns);
        });
    });

    group.finish();
}

/// 다양한 텍스트 타입 성능
fn bench_tokenizer_text_types(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_text_types");

    // 일반 문장
    group.bench_function("general", |b| {
        let text = MEDIUM_SENTENCES[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 기술 문서
    group.bench_function("technical", |b| {
        let text = TECHNICAL_TEXTS[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 혼합 텍스트
    group.bench_function("mixed", |b| {
        let text = MIXED_TEXTS[0];
        group.throughput(Throughput::Bytes(text.len() as u64));

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 실제 사용 시나리오 시뮬레이션
fn bench_tokenizer_realistic_workload(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_realistic");

    // 소셜 미디어 분석 (짧은 문장 대량 처리)
    group.bench_function("social_media", |b| {
        let total_bytes: usize = SHORT_SENTENCES.iter().map(|s| s.len()).sum();

        b.iter(|| {
            let mut total_tokens = 0;
            for &text in SHORT_SENTENCES {
                let tokens = tokenizer.tokenize(black_box(text));
                total_tokens += tokens.len();
            }
            black_box(total_tokens);
        });
    });

    // 뉴스 기사 분석 (긴 문장)
    group.bench_function("news_article", |b| {
        let total_bytes: usize = LONG_SENTENCES.iter().map(|s| s.len()).sum();

        b.iter(|| {
            let mut total_tokens = 0;
            for &text in LONG_SENTENCES {
                let tokens = tokenizer.tokenize(black_box(text));
                total_tokens += tokens.len();
            }
            black_box(total_tokens);
        });
    });

    // 문서 검색 (명사 추출)
    group.bench_function("document_indexing", |b| {
        let total_bytes: usize = MEDIUM_SENTENCES.iter().map(|s| s.len()).sum();

        b.iter(|| {
            let mut total_nouns = 0;
            for &text in MEDIUM_SENTENCES {
                let nouns = tokenizer.nouns(black_box(text));
                total_nouns += nouns.len();
            }
            black_box(total_nouns);
        });
    });

    group.finish();
}

/// 메모리 할당 오버헤드 측정
fn bench_tokenizer_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_memory");

    // 토크나이저 생성 오버헤드
    group.bench_function("creation", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new().expect("Failed to create");
            black_box(tokenizer);
        });
    });

    // 토크나이저 재사용
    group.bench_function("reuse", |b| {
        let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
        let text = MEDIUM_SENTENCES[0];

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 처리량 측정 (문자/초, 토큰/초)
fn bench_tokenizer_throughput(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_throughput");

    // 대량 텍스트 처리
    group.bench_function("high_volume", |b| {
        // 모든 문장을 결합하여 대량 텍스트 생성
        let all_sentences: Vec<_> = SHORT_SENTENCES
            .iter()
            .chain(MEDIUM_SENTENCES.iter())
            .chain(LONG_SENTENCES.iter())
            .copied()
            .collect();

        let total_bytes: usize = all_sentences.iter().map(|s| s.len()).sum();

        b.iter(|| {
            let mut total_tokens = 0;
            for &text in &all_sentences {
                let tokens = tokenizer.tokenize(black_box(text));
                total_tokens += tokens.len();
            }
            black_box(total_tokens);
        });
    });

    group.finish();
}

/// 특수 케이스 처리
fn bench_tokenizer_edge_cases(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_edge_cases");

    // 빈 문자열
    group.bench_function("empty", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(""));
            black_box(tokens);
        });
    });

    // 단일 문자
    group.bench_function("single_char", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("가"));
            black_box(tokens);
        });
    });

    // 공백만
    group.bench_function("whitespace_only", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("     "));
            black_box(tokens);
        });
    });

    // 숫자만
    group.bench_function("numbers_only", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("1234567890"));
            black_box(tokens);
        });
    });

    // 영문만
    group.bench_function("english_only", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("Hello World"));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 비교 벤치마크용 - 미래 구현 대비
fn bench_tokenizer_comparison_baseline(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    let mut group = c.benchmark_group("tokenizer_baseline");
    group.sample_size(100);

    // 표준 테스트 문장들로 베이스라인 설정
    for (idx, &text) in MEDIUM_SENTENCES.iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("sentence", idx), text, |b, &text| {
            group.throughput(Throughput::Bytes(text.len() as u64));

            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenizer_basic,
    bench_tokenizer_batch,
    bench_tokenizer_scalability,
    bench_tokenizer_wakati,
    bench_tokenizer_pos,
    bench_tokenizer_nouns,
    bench_tokenizer_text_types,
    bench_tokenizer_realistic_workload,
    bench_tokenizer_memory,
    bench_tokenizer_throughput,
    bench_tokenizer_edge_cases,
    bench_tokenizer_comparison_baseline,
);

criterion_main!(benches);
