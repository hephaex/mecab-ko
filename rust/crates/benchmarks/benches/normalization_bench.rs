//! 정규화 성능 벤치마크
//!
//! 측정 항목:
//! - 정규화 활성화 vs 비활성화
//! - 다양한 정규화 옵션 조합
//! - 특수 문자 처리 오버헤드
//! - 유니코드 정규화 비용

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use mecab_ko_core::tokenizer::Tokenizer;

/// 정규화 활성화 vs 비활성화
fn bench_normalization_toggle(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalization_toggle");

    let text = "한국어 형태소 분석기는 自然語 處理의 핵심 기술입니다!!!";
    group.throughput(Throughput::Bytes(text.len() as u64));

    // 정규화 비활성화 (기본)
    group.bench_function("disabled", |b| {
        let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(text));
            black_box(tokens);
        });
    });

    // 정규화 활성화는 실제 구현에 따라 다름
    // 현재는 스텁이므로 플레이스홀더

    group.finish();
}

/// 다양한 텍스트 타입에서의 정규화 비용
fn bench_normalization_by_text_type(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_by_text_type");

    // 순수 한글 (정규화 불필요)
    let pure_korean = "한국어 형태소 분석기는 자연어 처리의 핵심 기술입니다";
    group.throughput(Throughput::Bytes(pure_korean.len() as u64));
    group.bench_function("pure_korean", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(pure_korean));
            black_box(tokens);
        });
    });

    // 한자 포함
    let with_hanja = "漢字가 포함된 텍스트入니다";
    group.throughput(Throughput::Bytes(with_hanja.len() as u64));
    group.bench_function("with_hanja", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(with_hanja));
            black_box(tokens);
        });
    });

    // 영문 혼합
    let mixed_english = "Korean과 English가 mixed된 text입니다";
    group.throughput(Throughput::Bytes(mixed_english.len() as u64));
    group.bench_function("mixed_english", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(mixed_english));
            black_box(tokens);
        });
    });

    // 특수문자 다수
    let special_chars = "!!!이것은??? ***특수문자***가 많은 텍스트입니다!!!";
    group.throughput(Throughput::Bytes(special_chars.len() as u64));
    group.bench_function("special_chars", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(special_chars));
            black_box(tokens);
        });
    });

    // 이모지 포함
    let with_emoji = "이모지 😀😃😄 포함 텍스트입니다 🎉🎊";
    group.throughput(Throughput::Bytes(with_emoji.len() as u64));
    group.bench_function("with_emoji", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(with_emoji));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 유니코드 정규화 (NFC, NFD, NFKC, NFKD)
fn bench_unicode_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_unicode");

    // Precomposed vs Decomposed 한글
    let nfc_text = "한글"; // NFC (precomposed)
    let nfd_text = "한글"; // NFD (decomposed) - 실제로는 다를 수 있음

    group.bench_function("nfc_hangul", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(nfc_text));
            black_box(tokens);
        });
    });

    group.bench_function("nfd_hangul", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(nfd_text));
            black_box(tokens);
        });
    });

    // 호환 문자
    let compat_text = "㎏㎖㎝"; // 호환 문자
    group.bench_function("compatibility_chars", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(compat_text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 대소문자 정규화
fn bench_case_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_case");

    let lowercase = "this is lowercase text";
    let uppercase = "THIS IS UPPERCASE TEXT";
    let mixed_case = "ThIs Is MiXeD CaSe TeXt";

    group.bench_function("lowercase", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(lowercase));
            black_box(tokens);
        });
    });

    group.bench_function("uppercase", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(uppercase));
            black_box(tokens);
        });
    });

    group.bench_function("mixed_case", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(mixed_case));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 공백 정규화
fn bench_whitespace_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_whitespace");

    let single_space = "단어 사이 공백";
    let multiple_spaces = "단어    사이    여러    공백";
    let mixed_whitespace = "단어\t사이\n다양한\r\n공백";

    group.bench_function("single_space", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(single_space));
            black_box(tokens);
        });
    });

    group.bench_function("multiple_spaces", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(multiple_spaces));
            black_box(tokens);
        });
    });

    group.bench_function("mixed_whitespace", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(mixed_whitespace));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 숫자 정규화
fn bench_number_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_numbers");

    let arabic = "2024년 1월 15일";
    let fullwidth = "２０２４年 １月 １５日";
    let mixed = "2024년 １월 15日";

    group.bench_function("arabic_numerals", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(arabic));
            black_box(tokens);
        });
    });

    group.bench_function("fullwidth_numerals", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(fullwidth));
            black_box(tokens);
        });
    });

    group.bench_function("mixed_numerals", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(mixed));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 구두점 정규화
fn bench_punctuation_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_punctuation");

    let standard = "안녕하세요. 반갑습니다!";
    let fullwidth = "안녕하세요。反갑습니다!";
    let mixed = "안녕하세요. 반갑습니다!";

    group.bench_function("standard_punctuation", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(standard));
            black_box(tokens);
        });
    });

    group.bench_function("fullwidth_punctuation", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(fullwidth));
            black_box(tokens);
        });
    });

    group.bench_function("mixed_punctuation", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(mixed));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 실제 웹 텍스트 정규화
fn bench_web_text_normalization(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_web_text");

    // HTML 엔티티
    let html_entities = "안녕하세요&nbsp;반갑습니다&lt;태그&gt;";

    // URL 포함
    let with_url = "웹사이트는 https://example.com 입니다";

    // 해시태그, 멘션
    let social_media = "#한국어 #형태소분석 @사용자 안녕하세요";

    group.bench_function("html_entities", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(html_entities));
            black_box(tokens);
        });
    });

    group.bench_function("with_url", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(with_url));
            black_box(tokens);
        });
    });

    group.bench_function("social_media", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(social_media));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 정규화 체인 (여러 정규화 단계)
fn bench_normalization_chain(c: &mut Criterion) {
    let tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_chain");

    // 모든 정규화가 필요한 복잡한 텍스트
    let complex_text = "!!!  ２０２４년    １월  １５일  !!!  \
                        漢字와  ENGLISH와  한글이  섞인  \
                        텍스트입니다  😀  \
                        https://example.com  #해시태그";

    group.throughput(Throughput::Bytes(complex_text.len() as u64));

    group.bench_function("complex_text", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(complex_text));
            black_box(tokens);
        });
    });

    group.finish();
}

/// 정규화 오버헤드 측정
fn bench_normalization_overhead(c: &mut Criterion) {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");
    let mut group = c.benchmark_group("normalization_overhead");

    // 정규화 불필요 (깨끗한 텍스트)
    let clean_text = "한국어 형태소 분석기";

    // 정규화 필요 (더러운 텍스트)
    let dirty_text = "!!!한국어!!!   형태소   분석기!!!";

    group.bench_function("clean_text", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(clean_text));
            black_box(tokens);
        });
    });

    group.bench_function("dirty_text", |b| {
        b.iter(|| {
            let tokens = tokenizer.tokenize(black_box(dirty_text));
            black_box(tokens);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_normalization_toggle,
    bench_normalization_by_text_type,
    bench_unicode_normalization,
    bench_case_normalization,
    bench_whitespace_normalization,
    bench_number_normalization,
    bench_punctuation_normalization,
    bench_web_text_normalization,
    bench_normalization_chain,
    bench_normalization_overhead,
);

criterion_main!(benches);
