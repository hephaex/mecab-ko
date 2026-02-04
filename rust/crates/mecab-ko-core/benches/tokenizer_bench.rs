//! # 토크나이저 벤치마크
//!
//! 다양한 입력 크기와 텍스트 유형에 대한 형태소 분석 성능 벤치마크

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mecab_ko_core::Tokenizer;

/// 벤치마크용 샘플 텍스트
mod sample_texts {
    /// 짧은 텍스트 (10자 내외)
    pub const SHORT: &str = "안녕하세요";

    /// 중간 텍스트 (50자 내외)
    pub const MEDIUM: &str =
        "오늘 날씨가 정말 좋습니다. 산책하기 딱 좋은 날씨네요. 기분이 좋아집니다.";

    /// 긴 텍스트 (200자 내외)
    pub const LONG: &str = "\
        인공지능 기술의 발전으로 자연어 처리 분야가 급격히 성장하고 있습니다. \
        특히 한국어 형태소 분석은 교착어의 특성상 매우 복잡한 과정을 거치게 됩니다. \
        MeCab은 일본어 형태소 분석기로 시작되었지만, 한국어에도 적용되어 \
        높은 성능과 정확도를 보여주고 있습니다. 이 프로젝트는 Rust로 재작성하여 \
        메모리 안전성과 성능을 동시에 달성하는 것을 목표로 합니다.";

    /// 뉴스 스타일 텍스트
    pub const NEWS: &str = "\
        서울 강남구청은 오늘 오전 10시 기자회견을 열고 \
        새로운 도시재생 프로젝트를 발표했습니다. \
        이번 프로젝트는 총 500억원의 예산이 투입되며, \
        향후 3년간 진행될 예정입니다. \
        구청장은 \"주민들의 삶의 질 향상에 기여할 것\"이라고 밝혔습니다.";

    /// SNS 스타일 텍스트 (구어체, 이모티콘)
    pub const SOCIAL: &str = "\
        ㅋㅋㅋ 오늘 진짜 대박이었어 ㅎㅎ \
        친구들이랑 맛집 갔는데 완전 JMT!! \
        사진 찍어서 인스타에 올렸더니 \
        좋아요 100개 넘게 받음 ㅠㅠ \
        너무 행복해 ㅜㅜ";

    /// 기술 문서 스타일 텍스트
    pub const TECHNICAL: &str = "\
        Rust 프로그래밍 언어는 메모리 안전성을 보장하면서도 \
        C/C++에 버금가는 성능을 제공합니다. \
        소유권(ownership) 시스템을 통해 컴파일 타임에 \
        메모리 버그를 방지하며, 데이터 레이스(data race)도 \
        컴파일러 수준에서 차단합니다. \
        제로 코스트 추상화(zero-cost abstraction) 원칙을 따르므로 \
        고수준 추상화를 사용해도 런타임 오버헤드가 없습니다.";

    /// 법률 문서 스타일 텍스트
    pub const LEGAL: &str = "\
        제1조(목적) 이 법은 국민의 기본권을 보장하고 \
        민주적 기본질서를 확립하기 위하여 필요한 사항을 규정함을 목적으로 한다. \
        제2조(정의) 이 법에서 사용하는 용어의 뜻은 다음과 같다. \
        1. \"국민\"이란 대한민국의 국적을 가진 자를 말한다. \
        2. \"공공기관\"이란 국가기관 및 지방자치단체를 말한다.";

    /// 1KB 크기 텍스트 생성
    pub fn generate_1kb() -> String {
        LONG.repeat(5) // 약 1KB
    }

    /// 10KB 크기 텍스트 생성
    pub fn generate_10kb() -> String {
        LONG.repeat(50) // 약 10KB
    }

    /// 100KB 크기 텍스트 생성
    pub fn generate_100kb() -> String {
        LONG.repeat(500) // 약 100KB
    }

    /// 1MB 크기 텍스트 생성
    pub fn generate_1mb() -> String {
        LONG.repeat(5000) // 약 1MB
    }
}

/// 기본 토크나이저 성능 벤치마크
fn bench_tokenize_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_basic");

    // 토크나이저 생성이 실패할 경우 벤치마크 건너뛰기
    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    group.bench_function("short", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(sample_texts::SHORT));
            black_box(tokens)
        })
    });

    group.bench_function("medium", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(sample_texts::MEDIUM));
            black_box(tokens)
        })
    });

    group.bench_function("long", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(sample_texts::LONG));
            black_box(tokens)
        })
    });

    group.finish();
}

/// 입력 크기별 처리량 벤치마크
fn bench_throughput_by_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput_by_size");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    let sizes = vec![
        ("1KB", sample_texts::generate_1kb()),
        ("10KB", sample_texts::generate_10kb()),
        ("100KB", sample_texts::generate_100kb()),
    ];

    for (name, text) in sizes {
        let byte_size = text.len();
        group.throughput(Throughput::Bytes(byte_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens)
            })
        });
    }

    group.finish();
}

/// 텍스트 유형별 벤치마크
fn bench_by_text_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("by_text_type");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    let text_types = vec![
        ("news", sample_texts::NEWS),
        ("social", sample_texts::SOCIAL),
        ("technical", sample_texts::TECHNICAL),
        ("legal", sample_texts::LEGAL),
    ];

    for (name, text) in text_types {
        group.bench_with_input(BenchmarkId::from_parameter(name), &text, |b, text| {
            b.iter(|| {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens)
            })
        });
    }

    group.finish();
}

/// Wakati 모드 벤치마크
fn bench_wakati(c: &mut Criterion) {
    let mut group = c.benchmark_group("wakati");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    group.bench_function("medium", |b| {
        b.iter(|| {
            let surfaces = tokenizer.wakati(black_box(sample_texts::MEDIUM));
            black_box(surfaces)
        })
    });

    group.finish();
}

/// 명사 추출 벤치마크
fn bench_nouns(c: &mut Criterion) {
    let mut group = c.benchmark_group("nouns");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    group.bench_function("medium", |b| {
        b.iter(|| {
            let nouns = tokenizer.nouns(black_box(sample_texts::MEDIUM));
            black_box(nouns)
        })
    });

    group.finish();
}

/// 품사 태깅 벤치마크
fn bench_pos(c: &mut Criterion) {
    let mut group = c.benchmark_group("pos");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    group.bench_function("medium", |b| {
        b.iter(|| {
            let pos_tags = tokenizer.pos(black_box(sample_texts::MEDIUM));
            black_box(pos_tags)
        })
    });

    group.finish();
}

/// 토크나이저 생성 오버헤드 벤치마크
fn bench_tokenizer_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenizer_creation");

    group.bench_function("new", |b| {
        b.iter(|| {
            let tokenizer = Tokenizer::new();
            black_box(tokenizer)
        })
    });

    group.finish();
}

/// 연속 분석 벤치마크 (Lattice 재사용 효과 측정)
fn bench_consecutive_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("consecutive_analysis");

    let Ok(mut tokenizer) = Tokenizer::new() else {
        eprintln!("Warning: Skipping benchmark - Tokenizer creation failed");
        return;
    };

    let texts = vec![
        sample_texts::SHORT,
        sample_texts::MEDIUM,
        sample_texts::LONG,
        sample_texts::NEWS,
        sample_texts::SOCIAL,
    ];

    group.bench_function("5_texts", |b| {
        b.iter(|| {
            for text in &texts {
                let tokens = tokenizer.tokenize(black_box(text));
                black_box(tokens);
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenize_basic,
    bench_throughput_by_size,
    bench_by_text_type,
    bench_wakati,
    bench_nouns,
    bench_pos,
    bench_tokenizer_creation,
    bench_consecutive_analysis,
);

criterion_main!(benches);
