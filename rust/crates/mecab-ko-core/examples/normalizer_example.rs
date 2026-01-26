//! # Foreign Word Normalization Example
//!
//! 외래어 표기 정규화 모듈 사용 예제

use mecab_ko_core::normalizer::{NormalizationConfig, Normalizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MeCab-Ko Foreign Word Normalization Example ===\n");

    // 1. 기본 정규화기 생성
    println!("1. Creating normalizer with default config...");
    let normalizer = Normalizer::default()?;
    println!("   ✓ Normalizer created\n");

    // 2. 기본 정규화
    println!("2. Basic normalization:");
    let test_words = vec![
        ("코피", "커피"),
        ("케익", "케이크"),
        ("소프트웨아", "소프트웨어"),
        ("라이브러이", "라이브러리"),
        ("디렉터리", "디렉토리"),
    ];

    for (variant, expected) in test_words {
        let normalized = normalizer.normalize(variant);
        println!(
            "   {} → {} {}",
            variant,
            normalized,
            if normalized == expected { "✓" } else { "✗" }
        );
    }
    println!();

    // 3. 변이형 조회
    println!("3. Getting variants:");
    let standards = vec!["커피", "케이크", "쿠버네티스"];

    for standard in standards {
        let variants = normalizer.get_variants(standard);
        println!("   {} → {:?}", standard, variants);
    }
    println!();

    // 4. 변이형 여부 확인
    println!("4. Checking if words are variants:");
    let pairs = vec![
        ("커피", "코피", true),
        ("케이크", "케익", true),
        ("커피", "라면", false),
        ("소프트웨어", "소프트웨아", true),
    ];

    for (word1, word2, expected) in pairs {
        let is_variant = normalizer.is_variant(word1, word2);
        println!(
            "   {} ↔ {} : {} {}",
            word1,
            word2,
            is_variant,
            if is_variant == expected { "✓" } else { "✗" }
        );
    }
    println!();

    // 5. 발음 유사도 계산
    println!("5. Phonetic similarity:");
    let similarity_pairs = vec![
        ("커피", "커피"),
        ("커피", "코피"),
        ("케이크", "케익"),
        ("커피", "라면"),
    ];

    for (word1, word2) in similarity_pairs {
        let similarity = normalizer.phonetic_similarity(word1, word2);
        println!("   {} ↔ {} : {:.2}", word1, word2, similarity);
    }
    println!();

    // 6. IT 용어 정규화
    println!("6. IT terminology normalization:");
    let it_terms = vec![
        "쿠베르네테스",
        "알고리듬",
        "데이타베이스",
        "네트웍",
        "라우타",
    ];

    for term in it_terms {
        let normalized = normalizer.normalize(term);
        println!("   {} → {}", term, normalized);
    }
    println!();

    // 7. 커스텀 설정
    println!("7. Custom configuration:");
    let mut custom_config = NormalizationConfig::default();
    custom_config.min_confidence = 0.9;
    custom_config.vowel_variation = false;

    let custom_normalizer = Normalizer::new(custom_config)?;
    println!("   ✓ Custom normalizer created with min_confidence=0.9\n");

    // 8. 생성된 변이형 테스트
    println!("8. Generated variants (rule-based):");
    let words_for_generation = vec!["커피", "소프트웨어"];

    for word in words_for_generation {
        let variants = normalizer.get_variants(word);
        println!("   {} has {} variant(s)", word, variants.len());
        for variant in variants.iter().take(5) {
            println!("     - {}", variant);
        }
    }
    println!();

    println!("=== Example completed successfully! ===");

    Ok(())
}
