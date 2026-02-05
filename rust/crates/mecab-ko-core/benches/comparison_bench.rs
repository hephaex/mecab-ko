//! # 경쟁 라이브러리 비교 벤치마크
//!
//! 다양한 분석 패턴에 대한 성능 비교
//! 실제 경쟁 라이브러리가 없으므로, 다양한 내부 구현 방식을 비교

#![allow(clippy::semicolon_if_nothing_returned, missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mecab_ko_core::Tokenizer;

/// 분석 모드별 성능 비교
fn bench_analysis_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_modes");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    let text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다";

    // 전체 토큰화
    group.bench_function("full_tokenize", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    // Wakati (표면형만)
    group.bench_function("wakati", |b| {
        b.iter(|| {
            let surfaces = tokenizer.wakati(black_box(text));
            black_box(surfaces)
        })
    });

    // 명사 추출
    group.bench_function("nouns", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns)
        })
    });

    // 품사 태깅
    group.bench_function("pos", |b| {
        b.iter(|| {
            let pos_tags = tokenizer.pos(black_box(text));
            black_box(pos_tags)
        })
    });

    // Morphs (Wakati와 동일)
    group.bench_function("morphs", |b| {
        b.iter(|| {
            let morphs = tokenizer.morphs(black_box(text));
            black_box(morphs)
        })
    });

    group.finish();
}

/// 다양한 언어적 특성별 성능
fn bench_linguistic_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("linguistic_features");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    // 교착어 특성 (조사 많음)
    group.bench_function("agglutinative", |b| {
        let text = "아버지가방에들어가신다고말씀하셨습니다";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    // 복합명사
    group.bench_function("compound_nouns", |b| {
        let text = "자연어처리기술개발연구소";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    // 외래어
    group.bench_function("foreign_words", |b| {
        let text = "컴퓨터 프로그래밍 언어 파이썬";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    // 숫자와 한글 혼합
    group.bench_function("mixed_numbers", |b| {
        let text = "2024년 1월 15일 오후 3시 30분";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    // 영어와 한글 혼합
    group.bench_function("mixed_english", |b| {
        let text = "Rust 프로그래밍 언어로 MeCab-Ko를 구현합니다";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens)
        })
    });

    group.finish();
}

/// 실제 사용 시나리오별 성능
fn bench_real_world_scenarios(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_scenarios");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    // 검색 쿼리 (짧고 빠른 응답 필요)
    group.bench_function("search_query", |b| {
        let queries = vec![
            "맛집 추천",
            "서울 날씨",
            "영화 예매",
            "책 리뷰",
            "여행지 정보",
        ];
        b.iter(|| {
            for query in &queries {
                let tokens = tokenizer.tokenize(black_box(query));
                black_box(tokens);
            }
        })
    });

    // 문서 색인 (대량 처리)
    group.bench_function("document_indexing", |b| {
        let document = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다. \
                       특히 한국어 형태소 분석은 교착어의 특성상 매우 복잡한 과정을 거치게 됩니다."
            .repeat(10);
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(&document));
            black_box(tokens)
        })
    });

    // 실시간 채팅 분석 (짧은 메시지 반복)
    group.bench_function("chat_messages", |b| {
        let messages = vec![
            "안녕하세요",
            "반갑습니다",
            "날씨가 좋네요",
            "오늘 뭐하세요?",
            "저녁 먹었어요?",
        ];
        b.iter(|| {
            for msg in &messages {
                let tokens = tokenizer.tokenize(black_box(msg));
                black_box(tokens);
            }
        })
    });

    // 감성 분석 (형용사/동사 추출)
    group.bench_function("sentiment_analysis", |b| {
        let text = "정말 좋은 영화였습니다. 배우들의 연기가 훌륭했고 \
                   스토리도 감동적이었습니다. 강력히 추천합니다.";
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            // 형용사/동사 필터링 시뮬레이션
            let filtered_count = tokens
                .iter()
                .filter(|t| t.pos.starts_with("VA") || t.pos.starts_with("VV"))
                .count();
            black_box(filtered_count)
        })
    });

    // 키워드 추출 (명사 중심)
    group.bench_function("keyword_extraction", |b| {
        let text = "인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다";
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(text));
            black_box(nouns)
        })
    });

    group.finish();
}

/// 엣지 케이스 처리 성능
fn bench_edge_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_cases");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    // 빈 문자열
    group.bench_function("empty_string", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(""));
            black_box(tokens)
        })
    });

    // 단일 문자
    group.bench_function("single_char", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("안"));
            black_box(tokens)
        })
    });

    // 특수문자만
    group.bench_function("special_chars", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("!@#$%^&*()"));
            black_box(tokens)
        })
    });

    // 숫자만
    group.bench_function("numbers_only", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("1234567890"));
            black_box(tokens)
        })
    });

    // 영어만
    group.bench_function("english_only", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box("Hello World"));
            black_box(tokens)
        })
    });

    // 매우 긴 단일 토큰
    group.bench_function("very_long_token", |b| {
        let long_word = "가".repeat(100);
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(&long_word));
            black_box(tokens)
        })
    });

    group.finish();
}

/// 배치 처리 vs 단일 처리
fn bench_batch_vs_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_vs_single");

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

    // 단일 토크나이저로 순차 처리
    group.bench_function("sequential_with_single_tokenizer", |b| {
        b.iter(|| {
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            }
        })
    });

    // 텍스트 합쳐서 한 번에 처리
    group.bench_function("combined_text", |b| {
        let combined = texts.join(" ");
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(&combined));
            black_box(tokens)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_analysis_modes,
    bench_linguistic_features,
    bench_real_world_scenarios,
    bench_edge_cases,
    bench_batch_vs_single,
);

criterion_main!(benches);
